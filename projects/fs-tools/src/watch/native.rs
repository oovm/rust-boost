//! Platform change-notification backends.
//!
//! Windows is the only host implemented so far. Linux and macOS still reject
//! [`WatchBackend::Native`](super::WatchBackend::Native) with
//! [`WatchError::Unsupported`](super::WatchError::Unsupported); callers there must select the
//! polling backend explicitly.

use std::sync::{Arc, Mutex, mpsc::Receiver};

use super::{
    emitter::Emitter,
    worker::{Command, DeliveryState},
};

#[cfg(windows)]
mod windows;

#[cfg(windows)]
pub(super) fn run(commands: Receiver<Command>, emitter: Arc<Emitter>, shared: Arc<Mutex<DeliveryState>>) {
    windows::run(commands, emitter, shared);
}

#[cfg(not(windows))]
pub(super) fn run(commands: Receiver<Command>, emitter: Arc<Emitter>, shared: Arc<Mutex<DeliveryState>>) {
    unsupported::run(commands, emitter, shared);
}

#[cfg(not(windows))]
mod unsupported {
    use std::sync::{Arc, Mutex, mpsc::Receiver};

    use super::super::{
        emitter::Emitter,
        worker::{Command, DeliveryState},
    };

    /// Never reached: construction rejects the native backend before the worker starts.
    pub(super) fn run(_: Receiver<Command>, _: Arc<Emitter>, _: Arc<Mutex<DeliveryState>>) {}
}
