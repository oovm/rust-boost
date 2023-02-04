use std::{
    fs::{File, OpenOptions},
    io,
    path::{Path, PathBuf},
};

use super::unique;

/// A temporary file removed on drop unless [`Self::keep`] is called.
pub struct NamedFile {
    file: Option<File>,
    path: PathBuf,
    keep: bool,
}

impl NamedFile {
    /// Returns the path of this temporary file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns a shared handle to the open file.
    pub fn file(&self) -> &File {
        self.file.as_ref().expect("temporary file handle")
    }

    /// Returns the open file and delete the path on drop.
    pub fn into_file(mut self) -> File {
        self.keep = false;
        self.file.take().expect("temporary file handle")
    }

    /// Keep the file after this value is dropped.
    pub fn keep(mut self) -> (File, PathBuf) {
        self.keep = true;
        (self.file.take().expect("temporary file handle"), std::mem::take(&mut self.path))
    }
}

impl Drop for NamedFile {
    fn drop(&mut self) {
        drop(self.file.take());
        if !self.keep {
            let _ = crate::remove_file(&self.path);
        }
    }
}

impl std::ops::Deref for NamedFile {
    type Target = File;

    fn deref(&self) -> &Self::Target {
        self.file()
    }
}

impl std::ops::DerefMut for NamedFile {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.file.as_mut().expect("temporary file handle")
    }
}

/// Create an anonymous temporary file.
///
/// The file is unlinked from the filesystem while remaining open, or deleted on close on
/// platforms that require it.
pub fn create_file() -> io::Result<File> {
    create_file_in(&super::env::dir_path()?)
}

/// Create an anonymous temporary file under `dir`.
pub fn create_file_in(dir: impl AsRef<Path>) -> io::Result<File> {
    let dir = dir.as_ref();
    for _ in 0..128 {
        let path = dir.join(unique::prefix("f"));
        match open_anonymous(&path) {
            Ok(file) => return Ok(file),
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(err),
        }
    }
    Err(io::Error::new(io::ErrorKind::AlreadyExists, "failed to allocate a unique temporary file"))
}

/// Create a named temporary file under [`super::env::dir_path`].
pub fn create_named() -> io::Result<NamedFile> {
    create_named_in(super::env::dir_path()?)
}

/// Create a named temporary file under `dir`.
pub fn create_named_in(dir: impl AsRef<Path>) -> io::Result<NamedFile> {
    let dir = dir.as_ref();
    for _ in 0..128 {
        let path = dir.join(unique::prefix("n"));
        match OpenOptions::new().read(true).write(true).create_new(true).open(&path) {
            Ok(file) => return Ok(NamedFile { file: Some(file), path, keep: false }),
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(err),
        }
    }
    Err(io::Error::new(io::ErrorKind::AlreadyExists, "failed to allocate a unique named temporary file"))
}

fn open_anonymous(path: &Path) -> io::Result<File> {
    #[cfg(any(unix, target_os = "wasi"))]
    {
        let file = OpenOptions::new().read(true).write(true).create_new(true).open(path)?;
        crate::remove_file(path)?;
        return Ok(file);
    }

    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_ATTRIBUTE_TEMPORARY: u32 = 0x0000_0100;
        const FILE_FLAG_DELETE_ON_CLOSE: u32 = 0x0400_0000;
        return OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .attributes(FILE_ATTRIBUTE_TEMPORARY)
            .custom_flags(FILE_FLAG_DELETE_ON_CLOSE)
            .open(path);
    }

    #[cfg(not(any(unix, target_os = "wasi", windows)))]
    {
        OpenOptions::new().read(true).write(true).create_new(true).open(path)
    }
}
