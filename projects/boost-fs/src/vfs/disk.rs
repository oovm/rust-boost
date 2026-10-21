use std::path::{Path, PathBuf};

use crate::metadata;

use super::{content::FileContent, error::VfsError, version::FileVersion};

/// Disk-backed file access rooted at an optional base directory.
#[derive(Debug, Clone)]
pub struct DiskFs {
    root: PathBuf,
}

impl DiskFs {
    /// Creates a disk layer with the given root. Relative paths resolve under `root`.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Resolves `path` against the configured root when relative.
    pub fn resolve(&self, path: &Path) -> PathBuf {
        if path.is_absolute() { path.to_path_buf() } else { self.root.join(path) }
    }

    /// Reads a file from disk and pairs the bytes with a metadata-derived version.
    pub fn read(&self, path: &Path) -> Result<FileContent, VfsError> {
        let resolved = self.resolve(path);
        if !resolved.exists() {
            return Err(VfsError::NotFound(resolved));
        }
        let meta = metadata::metadata(&resolved)?;
        if !meta.is_file() {
            return Err(VfsError::NotAFile(resolved));
        }
        let modified_nanos = meta.modified()?.duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(0);
        let bytes = crate::read(&resolved)?;
        Ok(FileContent { bytes, version: FileVersion::from_disk(modified_nanos, meta.len()) })
    }

    /// Returns whether `path` exists on disk.
    pub fn exists(&self, path: &Path) -> bool {
        self.resolve(path).exists()
    }

    /// Reads directory entries as resolved paths.
    pub fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>, VfsError> {
        let resolved = self.resolve(path);
        if !resolved.exists() {
            return Err(VfsError::NotFound(resolved));
        }
        if !resolved.is_dir() {
            return Err(VfsError::NotADirectory(resolved));
        }
        let mut entries = Vec::new();
        for entry in crate::read_dir(&resolved)? {
            entries.push(entry?.path());
        }
        Ok(entries)
    }
}
