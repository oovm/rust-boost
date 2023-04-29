//! Events delivered by a [`Watcher`](super::Watcher).
//!
//! A rename is reported with the information the platform actually provides. The event model is
//! therefore not uniform across hosts: consumers must treat [`WatchEvent::Renamed`] as
//! [`WatchEvent::Removed`] plus [`WatchEvent::Created`].

use std::path::PathBuf;

/// A change observed under a watched root.
#[derive(Debug)]
pub enum WatchEvent {
    /// A path was created.
    Created(PathBuf),
    /// A path's contents or metadata changed.
    Modified(PathBuf),
    /// A path was removed.
    Removed(PathBuf),
    /// A path was renamed, with both sides reported by the platform.
    ///
    /// Hosts that cannot pair a rename deliver [`Self::Removed`] and [`Self::Created`] instead, so
    /// consumers must handle both forms.
    Renamed {
        /// The path before the rename.
        from: PathBuf,
        /// The path after the rename.
        to: PathBuf,
    },
    /// The subtree may differ from the events already delivered.
    ///
    /// The consumer must re-read the subtree instead of assuming the event stream is complete.
    Rescan {
        /// The root whose state is uncertain.
        root: PathBuf,
        /// Why the rescan is required.
        reason: RescanReason,
    },
    /// A backend error.
    ///
    /// Errors other than [`super::WatchError::Fatal`] do not stop the watcher.
    Error(super::WatchError),
}

/// Why a [`WatchEvent::Rescan`] was emitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RescanReason {
    /// The event queue overflowed and events were dropped.
    Overflow,
    /// A new directory was registered and its contents may predate the subscription.
    NewDirectory,
    /// A rename was observed without a reliable pair, so both sides are uncertain.
    UnpairedRename,
}
