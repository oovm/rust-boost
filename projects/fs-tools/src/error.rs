use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
    io,
    path::{Path, PathBuf},
};

/// An error produced while recursively walking a directory tree.
#[derive(Debug)]
pub struct WalkError {
    path: Option<PathBuf>,
    depth: usize,
    kind: WalkErrorKind,
}

#[derive(Debug)]
enum WalkErrorKind {
    Io(io::Error),
    Loop { ancestor: PathBuf },
}

impl WalkError {
    /// Create an I/O error at the given path and depth.
    pub(crate) fn io(path: impl Into<PathBuf>, depth: usize, err: io::Error) -> Self {
        Self { path: Some(path.into()), depth, kind: WalkErrorKind::Io(err) }
    }

    /// Create a symlink loop error.
    pub(crate) fn loop_at(path: impl Into<PathBuf>, depth: usize, ancestor: PathBuf) -> Self {
        Self { path: Some(path.into()), depth, kind: WalkErrorKind::Loop { ancestor } }
    }

    /// Returns the path associated with this error if one exists.
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Returns the depth at which this error occurred relative to the root.
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Returns the underlying I/O error when the failure was I/O related.
    pub fn io_error(&self) -> Option<&io::Error> {
        match &self.kind {
            WalkErrorKind::Io(err) => Some(err),
            WalkErrorKind::Loop { .. } => None,
        }
    }

    /// Returns the ancestor path when a symlink loop was detected.
    pub fn loop_ancestor(&self) -> Option<&Path> {
        match &self.kind {
            WalkErrorKind::Loop { ancestor } => Some(ancestor),
            WalkErrorKind::Io(_) => None,
        }
    }
}

impl Display for WalkError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.kind {
            WalkErrorKind::Io(err) => {
                if let Some(path) = &self.path {
                    write!(f, "failed to access {}: {}", path.display(), err)
                }
                else {
                    write!(f, "{}", err)
                }
            }
            WalkErrorKind::Loop { ancestor } => {
                if let Some(path) = &self.path {
                    write!(f, "loop detected at {} via ancestor {}", path.display(), ancestor.display())
                }
                else {
                    write!(f, "loop detected via ancestor {}", ancestor.display())
                }
            }
        }
    }
}

impl StdError for WalkError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match &self.kind {
            WalkErrorKind::Io(err) => Some(err),
            WalkErrorKind::Loop { .. } => None,
        }
    }
}

impl From<WalkError> for io::Error {
    fn from(value: WalkError) -> Self {
        match value.kind {
            WalkErrorKind::Io(err) => err,
            WalkErrorKind::Loop { ancestor } => {
                let path = value.path.map(|p| p.display().to_string()).unwrap_or_default();
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("loop detected at {} via ancestor {}", path, ancestor.display()),
                )
            }
        }
    }
}
