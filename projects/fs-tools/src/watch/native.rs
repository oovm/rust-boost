//! Platform change-notification backends.
//!
//! Windows (`ReadDirectoryChangesW`), Linux (`inotify`), and macOS (FSEvents) are implemented.
//! Other hosts still reject [`WatchBackend::Native`](super::WatchBackend::Native) with
//! [`WatchError::Unsupported`](super::WatchError::Unsupported); callers there must select the
//! polling backend explicitly.

use std::sync::{Arc, Mutex, mpsc::Receiver};

use super::{
    emitter::Emitter,
    worker::{Command, DeliveryState},
};

#[cfg(windows)]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(windows)]
pub(super) fn run(commands: Receiver<Command>, emitter: Arc<Emitter>, shared: Arc<Mutex<DeliveryState>>) {
    windows::run(commands, emitter, shared);
}

#[cfg(target_os = "linux")]
pub(super) fn run(commands: Receiver<Command>, emitter: Arc<Emitter>, shared: Arc<Mutex<DeliveryState>>) {
    linux::run(commands, emitter, shared);
}

#[cfg(target_os = "macos")]
pub(super) fn run(commands: Receiver<Command>, emitter: Arc<Emitter>, shared: Arc<Mutex<DeliveryState>>) {
    macos::run(commands, emitter, shared);
}

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
pub(super) fn run(commands: Receiver<Command>, emitter: Arc<Emitter>, shared: Arc<Mutex<DeliveryState>>) {
    unsupported::run(commands, emitter, shared);
}

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
mod unsupported {
    use std::sync::{Arc, Mutex, mpsc::Receiver};

    use super::super::{
        emitter::Emitter,
        worker::{Command, DeliveryState},
    };

    /// Never reached: construction rejects the native backend before the worker starts.
    pub(super) fn run(_: Receiver<Command>, _: Arc<Emitter>, _: Arc<Mutex<DeliveryState>>) {}
}
