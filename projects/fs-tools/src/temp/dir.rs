use std::{
    io,
    path::{Path, PathBuf},
};

use super::unique;

/// A temporary directory removed on drop unless [`Self::keep`] is called.
pub struct TempDir {
    path: PathBuf,
    keep: bool,
}

impl TempDir {
    /// Returns the absolute path of this temporary directory.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Delete the directory and prevent the destructor from running again.
    pub fn close(mut self) -> io::Result<()> {
        let path = self.path.clone();
        self.keep = true;
        crate::remove_dir_all(path)
    }

    /// Keep the directory after this value is dropped.
    pub fn keep(mut self) -> PathBuf {
        self.keep = true;
        std::mem::take(&mut self.path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        if !self.keep {
            let _ = crate::remove_dir_all(&self.path);
        }
    }
}

/// Create a temporary directory under [`super::env::dir_path`].
pub fn create_dir() -> io::Result<TempDir> {
    create_dir_in(super::env::dir_path()?)
}

/// Create a temporary directory under `base`.
pub fn create_dir_in(base: impl AsRef<Path>) -> io::Result<TempDir> {
    let base = base.as_ref();
    for _ in 0..128 {
        let path = base.join(format!("{}-dir", unique::prefix("d")));
        match crate::create_dir(&path) {
            Ok(()) => return Ok(TempDir { path, keep: false }),
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(err),
        }
    }
    Err(io::Error::new(io::ErrorKind::AlreadyExists, "failed to allocate a unique temporary directory"))
}
