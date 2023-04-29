//! Errors returned by a [`Watcher`](super::Watcher).

use std::{
    error::Error,
    fmt, io,
    path::{Path, PathBuf},
};

/// A failure while creating, controlling, or running a watcher.
#[derive(Debug)]
#[non_exhaustive]
pub enum WatchError {
    /// The path does not exist.
    NotFound(PathBuf),
    /// The path exists but is not a directory.
    NotDirectory(PathBuf),
    /// The path cannot be read.
    PermissionDenied(PathBuf),
    /// The path is already watched by this watcher.
    AlreadyWatched(PathBuf),
    /// The path is not watched by this watcher.
    NotWatched(PathBuf),
    /// The host's watch or descriptor limit was reached.
    LimitReached,
    /// The host cannot provide a native backend with the required semantics.
    Unsupported,
    /// A control method was called from the delivery callback.
    Reentrant,
    /// The watcher options are invalid.
    InvalidOptions(&'static str),
    /// A backend operation failed but the watcher may continue.
    Backend(io::Error),
    /// The backend cannot continue and the watcher must be rebuilt.
    Fatal(io::Error),
}

impl fmt::Display for WatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(path) => write!(f, "watch path does not exist: {}", path.display()),
            Self::NotDirectory(path) => write!(f, "watch path is not a directory: {}", path.display()),
            Self::PermissionDenied(path) => write!(f, "watch path cannot be read: {}", path.display()),
            Self::AlreadyWatched(path) => write!(f, "watch path is already subscribed: {}", path.display()),
            Self::NotWatched(path) => write!(f, "watch path is not subscribed: {}", path.display()),
            Self::LimitReached => f.write_str("the host watch limit was reached"),
            Self::Unsupported => f.write_str("the host does not provide a native watcher backend"),
            Self::Reentrant => f.write_str("a watcher control method was called from the delivery callback"),
            Self::InvalidOptions(reason) => write!(f, "invalid watcher options: {reason}"),
            Self::Backend(error) => write!(f, "watch backend failed: {error}"),
            Self::Fatal(error) => write!(f, "watch backend stopped: {error}"),
        }
    }
}

impl Error for WatchError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Backend(error) | Self::Fatal(error) => Some(error),
            _ => None,
        }
    }
}

/// Map an IO error at `path` to the public error model.
pub(super) fn path_error(path: &Path, error: io::Error) -> WatchError {
    match error.kind() {
        io::ErrorKind::NotFound => WatchError::NotFound(path.to_path_buf()),
        io::ErrorKind::PermissionDenied => WatchError::PermissionDenied(path.to_path_buf()),
        _ => WatchError::Backend(error),
    }
}

/// The watcher worker is no longer running.
pub(super) fn worker_stopped() -> WatchError {
    WatchError::Backend(io::Error::new(io::ErrorKind::BrokenPipe, "the watcher worker has stopped"))
}
