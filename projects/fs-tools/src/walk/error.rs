use std::{
    error::Error as StdError,
    fmt::{self, Display, Formatter},
    io,
    path::{Path, PathBuf},
};

/// An error produced while recursively walking a folder tree.
#[derive(Debug)]
pub struct Error {
    path: Option<PathBuf>,
    depth: usize,
    kind: ErrorKind,
}

#[derive(Debug)]
enum ErrorKind {
    Io(io::Error),
    Loop { ancestor: PathBuf },
}

impl Error {
    pub(crate) fn io(path: impl Into<PathBuf>, depth: usize, err: io::Error) -> Self {
        Self { path: Some(path.into()), depth, kind: ErrorKind::Io(err) }
    }

    pub(crate) fn loop_at(path: impl Into<PathBuf>, depth: usize, ancestor: PathBuf) -> Self {
        Self { path: Some(path.into()), depth, kind: ErrorKind::Loop { ancestor } }
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
            ErrorKind::Io(err) => Some(err),
            ErrorKind::Loop { .. } => None,
        }
    }

    /// Returns the ancestor path when a symlink loop was detected.
    pub fn loop_ancestor(&self) -> Option<&Path> {
        match &self.kind {
            ErrorKind::Loop { ancestor } => Some(ancestor),
            ErrorKind::Io(_) => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ErrorKind::Io(err) => {
                if let Some(path) = &self.path {
                    write!(f, "failed to access {}: {}", path.display(), err)
                }
                else {
                    write!(f, "{}", err)
                }
            }
            ErrorKind::Loop { ancestor } => {
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

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match &self.kind {
            ErrorKind::Io(err) => Some(err),
            ErrorKind::Loop { .. } => None,
        }
    }
}

impl From<Error> for io::Error {
    fn from(value: Error) -> Self {
        match value.kind {
            ErrorKind::Io(err) => err,
            ErrorKind::Loop { ancestor } => {
                let path = value.path.map(|p| p.display().to_string()).unwrap_or_default();
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("loop detected at {} via ancestor {}", path, ancestor.display()),
                )
            }
        }
    }
}
