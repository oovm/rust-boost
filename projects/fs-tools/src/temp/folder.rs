use std::{
    io,
    path::{Path, PathBuf},
};

use super::unique;

/// A temporary folder removed on drop unless [`Self::keep`] is called.
pub struct TempFolder {
    path: PathBuf,
    keep: bool,
}

impl TempFolder {
    /// Returns the absolute path of this temporary folder.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Delete the folder and prevent the destructor from running again.
    pub fn close(mut self) -> io::Result<()> {
        let path = self.path.clone();
        self.keep = true;
        crate::remove_folder_all(path)
    }

    /// Keep the folder after this value is dropped.
    pub fn keep(mut self) -> PathBuf {
        self.keep = true;
        std::mem::take(&mut self.path)
    }
}

impl Drop for TempFolder {
    fn drop(&mut self) {
        if !self.keep {
            let _ = crate::remove_folder_all(&self.path);
        }
    }
}

/// Create a temporary folder under [`super::env::folder_path`].
pub fn create_folder() -> io::Result<TempFolder> {
    create_folder_in(super::env::folder_path()?)
}

/// Create a temporary folder under `base`.
pub fn create_folder_in(base: impl AsRef<Path>) -> io::Result<TempFolder> {
    let base = base.as_ref();
    for _ in 0..128 {
        let path = base.join(format!("{}-folder", unique::prefix("d")));
        match crate::create_folder(&path) {
            Ok(()) => return Ok(TempFolder { path, keep: false }),
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(err),
        }
    }
    Err(io::Error::new(io::ErrorKind::AlreadyExists, "failed to allocate a unique temporary folder"))
}
