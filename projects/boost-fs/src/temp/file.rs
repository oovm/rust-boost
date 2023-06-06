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

    /// Returns the open file and give up the temporary name.
    ///
    /// The name belongs to this value, so consuming the value removes it and the handle keeps the
    /// bytes reachable without a path.
    pub fn into_file(mut self) -> File {
        self.keep = false;
        self.file.take().expect("temporary file handle")
    }

    /// Delete the file and prevent the destructor from removing it again.
    ///
    /// The open handle is closed before the path is removed, so a host that refuses to remove an
    /// open file still releases the name.
    pub fn close(mut self) -> io::Result<()> {
        let path = self.path.clone();
        self.keep = true;
        drop(self.file.take());
        crate::remove_file(path)
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
    create_file_in(&super::env::folder_path()?)
}

/// Create an anonymous temporary file under `folder`.
///
/// A relative `folder` is resolved against the current folder, so the candidate is not created
/// somewhere else when the current folder changes later.
pub fn create_file_in(folder: impl AsRef<Path>) -> io::Result<File> {
    let folder = super::env::absolute(folder.as_ref())?;
    for _ in 0..super::RETRY_LIMIT {
        let path = folder.join(unique::prefix("f")?);
        match open_anonymous(&path) {
            Ok(file) => return Ok(file),
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(err),
        }
    }
    Err(io::Error::new(io::ErrorKind::AlreadyExists, "failed to allocate a unique temporary file"))
}

/// Create a named temporary file under [`super::env::folder_path`].
pub fn create_named() -> io::Result<NamedFile> {
    create_named_in(super::env::folder_path()?)
}

/// Create a named temporary file under `folder`.
///
/// A relative `folder` is resolved against the current folder, so the returned path is absolute.
pub fn create_named_in(folder: impl AsRef<Path>) -> io::Result<NamedFile> {
    let folder = super::env::absolute(folder.as_ref())?;
    for _ in 0..super::RETRY_LIMIT {
        let path = folder.join(unique::prefix("n")?);
        match create_owned_file(&path) {
            Ok(file) => return Ok(NamedFile { file: Some(file), path, keep: false }),
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(err),
        }
    }
    Err(io::Error::new(io::ErrorKind::AlreadyExists, "failed to allocate a unique named temporary file"))
}

/// Create a file that no account other than the owner can read or write.
///
/// The mode is given at creation on a host that carries one, so the file is never briefly readable
/// by others between creation and a later permission change.
fn create_owned_file(path: &Path) -> io::Result<File> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        return OpenOptions::new().read(true).write(true).create_new(true).mode(super::OWNER_ONLY_FILE).open(path);
    }

    #[cfg(not(unix))]
    {
        OpenOptions::new().read(true).write(true).create_new(true).open(path)
    }
}

fn open_anonymous(path: &Path) -> io::Result<File> {
    #[cfg(any(unix, target_os = "wasi"))]
    {
        let file = create_owned_file(path)?;
        crate::remove_file(path)?;
        Ok(file)
    }

    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_ATTRIBUTE_TEMPORARY: u32 = 0x0000_0100;
        const FILE_FLAG_DELETE_ON_CLOSE: u32 = 0x0400_0000;
        OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .attributes(FILE_ATTRIBUTE_TEMPORARY)
            .custom_flags(FILE_FLAG_DELETE_ON_CLOSE)
            .open(path)
    }

    #[cfg(not(any(unix, target_os = "wasi", windows)))]
    {
        OpenOptions::new().read(true).write(true).create_new(true).open(path)
    }
}
