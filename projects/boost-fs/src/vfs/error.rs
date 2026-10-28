use std::{
    fmt::{self, Display, Formatter},
    io,
    path::PathBuf,
};

/// VFS access failure.
#[derive(Debug)]
pub enum VfsError {
    /// Path does not exist.
    NotFound(PathBuf),
    /// Path exists but is not a regular file.
    NotAFile(PathBuf),
    /// Path exists but is not a directory.
    NotADirectory(PathBuf),
    /// Underlying IO failure.
    Io(io::Error),
}

impl Display for VfsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound(path) => write!(f, "path not found: {path:?}"),
            Self::NotAFile(path) => write!(f, "path is not a file: {path:?}"),
            Self::NotADirectory(path) => write!(f, "path is not a directory: {path:?}"),
            Self::Io(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for VfsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for VfsError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}
