use std::path::{Path, PathBuf};

use super::{content::FileContent, disk::DiskFs, error::VfsError, overlay::Overlay, version::FileVersion};

/// Disk access with an in-memory overlay for unsaved buffers.
#[derive(Debug)]
pub struct LayeredFs {
    disk: DiskFs,
    overlay: Overlay,
}

impl LayeredFs {
    /// Creates a layered filesystem rooted at `root`.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { disk: DiskFs::new(root), overlay: Overlay::new() }
    }

    /// Returns the overlay handle for session updates.
    pub fn overlay(&self) -> &Overlay {
        &self.overlay
    }

    /// Returns the disk layer.
    pub fn disk(&self) -> &DiskFs {
        &self.disk
    }

    /// Resolves `path` against the disk root when relative.
    pub fn resolve(&self, path: &Path) -> PathBuf {
        self.disk.resolve(path)
    }

    /// Reads overlay content first, otherwise disk.
    pub fn read(&self, path: &Path) -> Result<FileContent, VfsError> {
        if self.overlay.is_removed(path) {
            return Err(VfsError::NotFound(self.resolve(path)));
        }
        if let Some(content) = self.overlay.read(path) {
            return Ok(content);
        }
        self.disk.read(path)
    }

    /// Writes unsaved content into the overlay.
    pub fn write_overlay(&self, path: PathBuf, bytes: Vec<u8>) -> FileVersion {
        self.overlay.write(path, bytes)
    }

    /// Removes a path from the overlay and hides the backing file until cleared.
    pub fn remove_overlay(&self, path: &Path) -> FileVersion {
        self.overlay.remove(path)
    }

    /// Drops overlay state for `path`.
    pub fn clear_overlay(&self, path: &Path) {
        self.overlay.clear(path)
    }

    /// Returns whether the path exists in overlay or on disk.
    pub fn exists(&self, path: &Path) -> bool {
        !self.overlay.is_removed(path) && (self.overlay.read(path).is_some() || self.disk.exists(path))
    }

    /// Lists directory entries from disk. Overlay-only files are not merged yet.
    pub fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>, VfsError> {
        self.disk.read_dir(path)
    }
}
