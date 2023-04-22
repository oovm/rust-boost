//! Atomic file publish helpers.
//!
//! Writes go to a same-directory temporary file first, then into place. The staging file is owned by
//! the publisher until the destination is published, so a failed publish removes it instead of
//! leaving it behind.

use std::{
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
};

use crate::temp::{NamedFile, create_named_in};

/// Whether an existing destination may be replaced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverwritePolicy {
    /// Never replace the destination.
    ///
    /// [`AtomicPublisher::commit`] links the staged file into place, which fails on an existing
    /// destination without touching it, so a destination that appears after [`AtomicPublisher::create`]
    /// is still reported instead of being replaced.
    RefuseExisting,
    /// Replace an existing destination.
    ///
    /// Replacing is a rename, so a reader either sees the previous file or the published one.
    Replace,
}

/// Options controlling durability and overwrite behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PublishOptions {
    /// How to handle an existing destination path.
    pub overwrite: OverwritePolicy,
    /// `fsync` the temporary file before rename.
    pub fsync_file: bool,
    /// `fsync` the destination folder after the destination is published.
    ///
    /// Platforms that do not hand out a folder handle (Windows) cannot flush the parent folder, so
    /// the option is a no-op there.
    pub fsync_parent: bool,
}

impl Default for PublishOptions {
    fn default() -> Self {
        Self { overwrite: OverwritePolicy::RefuseExisting, fsync_file: true, fsync_parent: true }
    }
}

/// In-progress atomic publish of one file in the destination directory.
pub struct AtomicPublisher {
    target: PathBuf,
    temp: Option<NamedFile>,
    options: PublishOptions,
    committed: bool,
}

impl AtomicPublisher {
    /// Create a temporary file next to `target` for staged writing.
    ///
    /// An existing destination is reported here so that a refused publish fails before any byte is
    /// staged. This check is an early report, not the guarantee: [`Self::commit`] decides whether the
    /// destination may be published at all.
    pub fn create(target: impl AsRef<Path>, options: PublishOptions) -> io::Result<Self> {
        let target = target.as_ref();
        let parent = target
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "destination has no parent directory"))?;
        if !parent.is_dir() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "destination parent directory does not exist"));
        }
        if target.exists() && options.overwrite == OverwritePolicy::RefuseExisting {
            return Err(io::Error::new(io::ErrorKind::AlreadyExists, "destination already exists"));
        }

        let temp = create_named_in(parent)?;
        Ok(Self { target: target.to_path_buf(), temp: Some(temp), options, committed: false })
    }

    /// Final destination path.
    pub fn target(&self) -> &Path {
        &self.target
    }

    /// Temporary path receiving staged bytes.
    pub fn staging_path(&self) -> &Path {
        self.temp().path()
    }

    /// Open handle to the staging file.
    pub fn file(&self) -> &File {
        self.temp().file()
    }

    /// Publish the staged file as the destination.
    ///
    /// [`OverwritePolicy::RefuseExisting`] publishes by linking, so a destination that appeared while
    /// the file was staged is refused with [`io::ErrorKind::AlreadyExists`] and left untouched.
    /// [`OverwritePolicy::Replace`] publishes by rename, so a reader sees either file and never a
    /// partial one.
    ///
    /// A failure before the destination is published removes the staging file. A failure after
    /// publishing means the destination is already visible, but the caller cannot assume it survived a
    /// crash, because flushing the file or its parent folder failed.
    pub fn commit(mut self) -> io::Result<PathBuf> {
        let temp = self.temp.take().expect("staging file");
        if self.options.fsync_file {
            temp.sync_all()?;
        }

        // The staging handle is closed before the file is published, so the rename and the link do not
        // depend on how the platform treats an open handle.
        let (file, staging_path) = temp.keep();
        drop(file);

        let published = match self.options.overwrite {
            OverwritePolicy::RefuseExisting => crate::hard_link(&staging_path, &self.target).and_then(|()| crate::remove_file(&staging_path)),
            OverwritePolicy::Replace => crate::rename(&staging_path, &self.target),
        };
        if let Err(error) = published {
            // `keep` gave up the ownership of the staging file, so a failed publish removes it here
            // instead of relying on the destructor.
            let _ = crate::remove_file(&staging_path);
            return Err(error);
        }

        self.committed = true;

        let target = self.target.clone();
        if self.options.fsync_parent {
            sync_parent(&target)?;
        }

        Ok(target)
    }

    fn temp(&self) -> &NamedFile {
        self.temp.as_ref().expect("staging file")
    }
}

impl Drop for AtomicPublisher {
    fn drop(&mut self) {
        if !self.committed {
            let _ = self.temp.take();
        }
    }
}

impl std::ops::Deref for AtomicPublisher {
    type Target = File;

    fn deref(&self) -> &Self::Target {
        self.file()
    }
}

impl std::ops::DerefMut for AtomicPublisher {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.temp.as_mut().expect("staging file")
    }
}

/// Write `bytes` to `target` atomically using default publish options.
pub fn publish_bytes(target: impl AsRef<Path>, bytes: &[u8]) -> io::Result<()> {
    publish_bytes_with_options(target, bytes, PublishOptions::default())
}

/// Write `bytes` to `target` atomically.
pub fn publish_bytes_with_options(target: impl AsRef<Path>, bytes: &[u8], options: PublishOptions) -> io::Result<()> {
    let mut publisher = AtomicPublisher::create(target, options)?;
    publisher.write_all(bytes)?;
    publisher.commit()?;
    Ok(())
}

/// Flush the folder that holds the destination.
///
/// A folder handle keeps the published name durable across a crash, so a failure to open or flush it
/// is reported instead of being ignored.
#[cfg(not(windows))]
fn sync_parent(path: &Path) -> io::Result<()> {
    let parent = path
        .parent()
        .filter(|dir| !dir.as_os_str().is_empty())
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "destination has no parent directory"))?;

    std::fs::File::open(parent)?.sync_all()
}

/// Windows does not let `std::fs::File` open a folder, so the parent folder cannot be flushed there.
#[cfg(windows)]
fn sync_parent(path: &Path) -> io::Result<()> {
    let _ = path;
    Ok(())
}
