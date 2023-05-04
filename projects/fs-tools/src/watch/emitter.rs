//! Bounded event delivery shared by every backend.
//!
//! The queue is a bounded [`SyncSender`], so a backend must never block on a full queue. Dropped
//! events are recorded per root and replayed as one [`WatchEvent::Rescan`] once the queue accepts
//! writes again.

use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{
        Arc, Mutex, MutexGuard, PoisonError,
        atomic::{AtomicBool, Ordering},
        mpsc::{SyncSender, TrySendError},
    },
};

use super::{
    RescanReason, WatchEvent,
    worker::{Message, RootId},
};

/// Per-root event delivery shared by the backend and its subscriber threads.
pub(super) struct Emitter {
    events: SyncSender<Message>,
    roots: Mutex<BTreeMap<RootId, PathBuf>>,
    pending: Mutex<BTreeMap<RootId, RescanReason>>,
    disconnected: AtomicBool,
}

impl Emitter {
    /// Wrap the bounded queue created by [`super::Watcher::new`].
    pub(super) fn new(events: SyncSender<Message>) -> Arc<Self> {
        Arc::new(Self {
            events,
            roots: Mutex::new(BTreeMap::new()),
            pending: Mutex::new(BTreeMap::new()),
            disconnected: AtomicBool::new(false),
        })
    }

    /// Start tracking `id` so a pending rescan can name its root.
    pub(super) fn register(&self, id: RootId, root: PathBuf) {
        lock(&self.roots).insert(id, root);
    }

    /// Stop tracking `id` and drop its pending rescan.
    pub(super) fn unregister(&self, id: RootId) {
        lock(&self.roots).remove(&id);
        lock(&self.pending).remove(&id);
    }

    /// Stop tracking every root.
    pub(super) fn clear(&self) {
        lock(&self.roots).clear();
        lock(&self.pending).clear();
    }

    /// Deliver one event, recording an overflow instead of blocking.
    pub(super) fn send(&self, id: RootId, event: WatchEvent) {
        match self.events.try_send(Message::Event { root: id, event }) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => self.mark(id, RescanReason::Overflow),
            Err(TrySendError::Disconnected(_)) => self.disconnected.store(true, Ordering::Relaxed),
        }
    }

    /// Record that `id` needs a rescan.
    ///
    /// One root keeps one pending reason per batch; a stronger reason replaces a weaker one.
    pub(super) fn mark(&self, id: RootId, reason: RescanReason) {
        let mut pending = lock(&self.pending);
        match pending.get_mut(&id) {
            Some(current) => *current = merge(*current, reason),
            None => {
                pending.insert(id, reason);
            }
        }
    }

    /// Deliver one pending rescan per affected root that the queue accepts.
    pub(super) fn flush(&self) {
        let pending = {
            let mut pending = lock(&self.pending);
            if pending.is_empty() {
                return;
            }
            std::mem::take(&mut *pending)
        };

        for (id, reason) in pending {
            let Some(root) = lock(&self.roots).get(&id).cloned()
            else {
                continue;
            };
            let event = WatchEvent::Rescan { root, reason };
            match self.events.try_send(Message::Event { root: id, event }) {
                Ok(()) => {}
                Err(TrySendError::Full(_)) => self.mark(id, reason),
                Err(TrySendError::Disconnected(_)) => {
                    self.disconnected.store(true, Ordering::Relaxed);
                    return;
                }
            }
        }
    }

    /// Whether the delivery thread is gone.
    pub(super) fn is_disconnected(&self) -> bool {
        self.disconnected.load(Ordering::Relaxed)
    }
}

fn merge(current: RescanReason, next: RescanReason) -> RescanReason {
    if priority(next) > priority(current) { next } else { current }
}

fn priority(reason: RescanReason) -> u8 {
    match reason {
        RescanReason::Overflow => 3,
        RescanReason::UnpairedRename => 2,
        RescanReason::NewDirectory => 1,
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}
