//! Atomic file publish helpers.
//!
//! Writes go to a same-directory temporary file first, then rename into place.

use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::temp::{create_named_in, NamedFile};

/// Whether an existing destination may be replaced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverwritePolicy {
    /// Fail with [`io::ErrorKind::AlreadyExists`] when the destination exists.
    RefuseExisting,
    /// Replace an existing destination atomically when the platform allows it.
    Replace,
}

/// Options controlling durability and overwrite behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PublishOptions {
    /// How to handle an existing destination path.
    pub overwrite: OverwritePolicy,
    /// `fsync` the temporary file before rename.
    pub fsync_file: bool,
    /// `fsync` the destination parent directory after rename.
    pub fsync_parent: bool,
}

impl Default for PublishOptions {
    fn default() -> Self {
        Self {
            overwrite: OverwritePolicy::RefuseExisting,
            fsync_file: true,
            fsync_parent: true,
        }
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
    pub fn create(target: impl AsRef<Path>, options: PublishOptions) -> io::Result<Self> {
        let target = target.as_ref();
        let parent = target
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "destination has no parent directory"))?;
        if !parent.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "destination parent directory does not exist",
            ));
        }
        if target.exists() && options.overwrite == OverwritePolicy::RefuseExisting {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "destination already exists",
            ));
        }

        let temp = create_named_in(parent)?;
        Ok(Self {
            target: target.to_path_buf(),
            temp: Some(temp),
            options,
            committed: false,
        })
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

    /// Rename the staging file into place.
    pub fn commit(mut self) -> io::Result<PathBuf> {
        if self.target.exists() && self.options.overwrite == OverwritePolicy::RefuseExisting {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "destination already exists",
            ));
        }

        let temp = self.temp.take().expect("staging file");
        if self.options.fsync_file {
            temp.sync_all()?;
        }

        let (_file, staging_path) = temp.keep();
        self.committed = true;

        crate::rename(&staging_path, &self.target)?;

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
pub fn publish_bytes_with_options(
    target: impl AsRef<Path>,
    bytes: &[u8],
    options: PublishOptions,
) -> io::Result<()> {
    let mut publisher = AtomicPublisher::create(target, options)?;
    publisher.write_all(bytes)?;
    publisher.commit()?;
    Ok(())
}

fn sync_parent(path: &Path) -> io::Result<()> {
    let parent = path
        .parent()
        .filter(|dir| !dir.as_os_str().is_empty())
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "destination has no parent directory"))?;

    if let Ok(dir) = std::fs::File::open(parent) {
        let _ = dir.sync_all();
    }

    Ok(())
}
