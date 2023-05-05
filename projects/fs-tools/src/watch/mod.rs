//! File system change observation.
//!
//! A [`Watcher`] owns a delivery thread and one backend thread. Subscriptions are recursive and
//! controlled with [`Watcher::watch`], [`Watcher::unwatch`], and [`Watcher::unwatch_all`].
//!
//! Events are best-effort state changes: duplicates are allowed, and a rename is only delivered as
//! [`WatchEvent::Renamed`] when the platform reports both sides. Consumers must treat that event as
//! [`WatchEvent::Removed`] plus [`WatchEvent::Created`].
//!
//! The event queue is bounded. When it overflows, events are dropped and a
//! [`WatchEvent::Rescan`] with [`RescanReason::Overflow`] is delivered for the affected root. The
//! consumer must re-read that subtree, because the stream alone is no longer complete.
//!
//! [`WatchBackend::Native`] is the default and implemented on Windows through
//! `ReadDirectoryChangesW`, on Linux through `inotify`, and on macOS through FSEvents. Other hosts
//! return [`WatchError::Unsupported`] for it; there [`WatchBackend::Polling`] must be requested
//! explicitly and reports only the net change between two samples.

mod emitter;
mod error;
mod event;
mod native;
mod options;
mod scan;
mod worker;

pub use error::WatchError;
pub use event::{RescanReason, WatchEvent};
pub use options::{DEFAULT_QUEUE_CAPACITY, MIN_POLL_INTERVAL, WatchBackend, WatchOptions};

use std::{
    io,
    path::Path,
    sync::{
        Arc, Mutex, MutexGuard, PoisonError,
        mpsc::{self, Receiver, Sender},
    },
    thread::{self, JoinHandle, ThreadId},
};

use emitter::Emitter;
use worker::{Command, DeliveryState, Message};

/// A recursive file system watcher.
///
/// Control methods must not be called from the delivery callback. Doing so returns
/// [`WatchError::Reentrant`] instead of deadlocking.
pub struct Watcher {
    commands: Sender<Command>,
    worker: Option<JoinHandle<()>>,
    delivery: Option<JoinHandle<()>>,
    delivery_thread: ThreadId,
    shared: Arc<Mutex<DeliveryState>>,
}

impl Watcher {
    /// Start watching with `callback` receiving every event.
    ///
    /// The callback runs on one delivery thread and must not call back into this watcher.
    pub fn new<F>(callback: F, options: WatchOptions) -> Result<Self, WatchError>
    where
        F: FnMut(WatchEvent) + Send + 'static,
    {
        let options = options.validate()?;
        let backend = options.get_backend();
        #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
        if matches!(backend, WatchBackend::Native) {
            return Err(WatchError::Unsupported);
        }

        let (command_sender, command_receiver) = mpsc::channel();
        let (event_sender, event_receiver) = mpsc::sync_channel(options.get_queue_capacity());
        let shared = Arc::new(Mutex::new(DeliveryState::new()));
        let emitter = Emitter::new(event_sender);

        let delivery_shared = Arc::clone(&shared);
        let delivery = thread::Builder::new()
            .name("fs-tools-watcher-delivery".into())
            .spawn(move || deliver(callback, event_receiver, delivery_shared))
            .map_err(WatchError::Backend)?;
        let delivery_thread = delivery.thread().id();

        let worker_shared = Arc::clone(&shared);
        let worker = match thread::Builder::new().name("fs-tools-watcher-worker".into()).spawn(move || match backend {
            WatchBackend::Polling { interval } => worker::run(command_receiver, emitter, worker_shared, interval),
            WatchBackend::Native => native::run(command_receiver, emitter, worker_shared),
        }) {
            Ok(worker) => worker,
            Err(error) => {
                drop(command_sender);
                let _ = delivery.join();
                return Err(WatchError::Backend(error));
            }
        };

        Ok(Self { commands: command_sender, worker: Some(worker), delivery: Some(delivery), delivery_thread, shared })
    }

    /// Subscribe `root` recursively.
    ///
    /// The root must exist and be a directory. New folders under it are subscribed automatically.
    pub fn watch(&mut self, root: impl AsRef<Path>) -> Result<(), WatchError> {
        self.ensure_not_reentrant()?;
        let path = root.as_ref().to_path_buf();
        let (ack, response) = mpsc::channel();
        self.commands.send(Command::Watch { path, ack }).map_err(|_| error::worker_stopped())?;
        response.recv().map_err(|_| error::worker_stopped())?
    }

    /// Remove `root` and every subscription below it.
    ///
    /// Events already queued for the root are discarded. A callback that is already running may
    /// finish before this method returns.
    pub fn unwatch(&mut self, root: impl AsRef<Path>) -> Result<(), WatchError> {
        self.ensure_not_reentrant()?;
        let path = root.as_ref().to_path_buf();
        let (ack, response) = mpsc::channel();
        self.commands.send(Command::Unwatch { path, ack }).map_err(|_| error::worker_stopped())?;
        response.recv().map_err(|_| error::worker_stopped())?
    }

    /// Remove every subscription.
    pub fn unwatch_all(&mut self) -> Result<(), WatchError> {
        self.ensure_not_reentrant()?;
        let (ack, response) = mpsc::channel();
        self.commands.send(Command::UnwatchAll { ack }).map_err(|_| error::worker_stopped())?;
        response.recv().map_err(|_| error::worker_stopped())?
    }

    /// Stop the backend and release every resource.
    ///
    /// Queued events are discarded. No callback starts after this returns.
    pub fn close(mut self) -> Result<(), WatchError> {
        self.shutdown()
    }

    fn ensure_not_reentrant(&self) -> Result<(), WatchError> {
        if thread::current().id() == self.delivery_thread { Err(WatchError::Reentrant) } else { Ok(()) }
    }

    fn shutdown(&mut self) -> Result<(), WatchError> {
        let Some(worker) = self.worker.take()
        else {
            return Ok(());
        };
        lock(&self.shared).closed = true;

        let mut result = self.request_shutdown();

        if thread::current().id() != self.delivery_thread {
            if worker.join().is_err() {
                result = result.and(Err(WatchError::Backend(io::Error::other("watcher worker panicked"))));
            }
            if let Some(delivery) = self.delivery.take()
                && delivery.join().is_err()
            {
                result = result.and(Err(WatchError::Backend(io::Error::other("watcher delivery panicked"))));
            }
        }
        result
    }

    fn request_shutdown(&self) -> Result<(), WatchError> {
        let (ack, response) = mpsc::channel();
        self.commands.send(Command::Shutdown { ack }).map_err(|_| error::worker_stopped())?;
        response.recv().map_err(|_| error::worker_stopped())?
    }
}

impl Drop for Watcher {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

fn deliver<F>(mut callback: F, events: Receiver<Message>, shared: Arc<Mutex<DeliveryState>>)
where
    F: FnMut(WatchEvent) + Send + 'static,
{
    while let Ok(Message::Event { root, event }) = events.recv() {
        let state = lock(&shared);
        if !state.closed && state.active.contains(&root) {
            callback(event);
        }
    }
}

fn lock(shared: &Arc<Mutex<DeliveryState>>) -> MutexGuard<'_, DeliveryState> {
    shared.lock().unwrap_or_else(PoisonError::into_inner)
}
