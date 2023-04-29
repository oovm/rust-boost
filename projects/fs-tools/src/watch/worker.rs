//! Subscription bookkeeping and the polling backend loop.

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, RecvTimeoutError, Sender, SyncSender, TrySendError},
    },
    time::Duration,
};

use super::{RescanReason, WatchError, WatchEvent, scan};

/// Identifier of one subscription root.
pub(super) type RootId = u64;

/// A control request handled by the worker thread.
pub(super) enum Command {
    Watch { path: PathBuf, ack: Sender<Result<(), WatchError>> },
    Unwatch { path: PathBuf, ack: Sender<Result<(), WatchError>> },
    UnwatchAll { ack: Sender<Result<(), WatchError>> },
    Shutdown { ack: Sender<Result<(), WatchError>> },
}

/// A message handed from the worker to the delivery thread.
pub(super) enum Message {
    Event { root: RootId, event: WatchEvent },
}

/// Subscription state shared with the delivery thread.
pub(super) struct DeliveryState {
    pub(super) active: HashSet<RootId>,
    pub(super) closed: bool,
}

impl DeliveryState {
    pub(super) fn new() -> Self {
        Self { active: HashSet::new(), closed: false }
    }
}

struct PollRoot {
    root: PathBuf,
    snapshot: BTreeMap<PathBuf, scan::Fingerprint>,
}

struct Worker {
    events: SyncSender<Message>,
    shared: Arc<Mutex<DeliveryState>>,
    roots: BTreeMap<RootId, PollRoot>,
    by_path: HashMap<PathBuf, RootId>,
    pending: BTreeMap<RootId, RescanReason>,
    next_id: RootId,
    disconnected: bool,
}

fn lock(shared: &Arc<Mutex<DeliveryState>>) -> std::sync::MutexGuard<'_, DeliveryState> {
    shared.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Run the backend loop until shutdown or until the watcher goes away.
pub(super) fn run(commands: Receiver<Command>, events: SyncSender<Message>, shared: Arc<Mutex<DeliveryState>>, interval: Duration) {
    let mut worker =
        Worker { events, shared, roots: BTreeMap::new(), by_path: HashMap::new(), pending: BTreeMap::new(), next_id: 0, disconnected: false };

    loop {
        match commands.recv_timeout(interval) {
            Ok(Command::Watch { path, ack }) => {
                let result = worker.watch(path);
                let _ = ack.send(result);
            }
            Ok(Command::Unwatch { path, ack }) => {
                let result = worker.unwatch(&path);
                let _ = ack.send(result);
            }
            Ok(Command::UnwatchAll { ack }) => {
                worker.unwatch_all();
                let _ = ack.send(Ok(()));
            }
            Ok(Command::Shutdown { ack }) => {
                let _ = ack.send(Ok(()));
                break;
            }
            Err(RecvTimeoutError::Timeout) => worker.poll(),
            Err(RecvTimeoutError::Disconnected) => break,
        }
        worker.flush_pending();
        if worker.disconnected {
            break;
        }
    }
}

impl Worker {
    fn watch(&mut self, path: PathBuf) -> Result<(), WatchError> {
        let root = scan::canonical_root(&path)?;
        if self.by_path.contains_key(&root) {
            return Err(WatchError::AlreadyWatched(path));
        }
        let snapshot = scan::scan_tree(&root).map_err(scan::ScanError::into_watch_error)?;

        let id = self.next_id;
        self.next_id += 1;
        lock(&self.shared).active.insert(id);
        self.by_path.insert(root.clone(), id);
        self.roots.insert(id, PollRoot { root, snapshot });
        Ok(())
    }

    fn unwatch(&mut self, path: &Path) -> Result<(), WatchError> {
        let key = scan::subscription_key(path);
        let id = self.by_path.remove(&key).ok_or_else(|| WatchError::NotWatched(path.to_path_buf()))?;
        self.roots.remove(&id);
        self.pending.remove(&id);
        lock(&self.shared).active.remove(&id);
        Ok(())
    }

    fn unwatch_all(&mut self) {
        self.roots.clear();
        self.by_path.clear();
        self.pending.clear();
        lock(&self.shared).active.clear();
    }

    fn poll(&mut self) {
        let ids: Vec<RootId> = self.roots.keys().copied().collect();
        for id in ids {
            let Some(root) = self.roots.get(&id).map(|entry| entry.root.clone())
            else {
                continue;
            };
            match scan::scan_tree(&root) {
                Ok(snapshot) => {
                    let Some(entry) = self.roots.get_mut(&id)
                    else {
                        continue;
                    };
                    let events = scan::diff(&entry.snapshot, &snapshot);
                    entry.snapshot = snapshot;
                    for event in events {
                        self.send(id, event);
                    }
                }
                Err(error) if error.is_missing_root(&root) => {
                    self.send(id, WatchEvent::Removed(root));
                    self.remove(id);
                }
                Err(error) => {
                    let error = error.into_watch_error();
                    self.send(id, WatchEvent::Error(error));
                }
            }
        }
    }

    fn remove(&mut self, id: RootId) {
        if let Some(entry) = self.roots.remove(&id) {
            self.by_path.remove(&entry.root);
        }
        self.pending.remove(&id);
        lock(&self.shared).active.remove(&id);
    }

    fn send(&mut self, id: RootId, event: WatchEvent) {
        match self.events.try_send(Message::Event { root: id, event }) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {
                self.pending.entry(id).or_insert(RescanReason::Overflow);
            }
            Err(TrySendError::Disconnected(_)) => self.disconnected = true,
        }
    }

    fn flush_pending(&mut self) {
        if self.pending.is_empty() {
            return;
        }
        let pending = std::mem::take(&mut self.pending);
        for (id, reason) in pending {
            let Some(root) = self.roots.get(&id).map(|entry| entry.root.clone())
            else {
                continue;
            };
            let event = WatchEvent::Rescan { root, reason };
            match self.events.try_send(Message::Event { root: id, event }) {
                Ok(()) => {}
                Err(TrySendError::Full(_)) => {
                    self.pending.insert(id, reason);
                }
                Err(TrySendError::Disconnected(_)) => {
                    self.disconnected = true;
                    return;
                }
            }
        }
    }
}
