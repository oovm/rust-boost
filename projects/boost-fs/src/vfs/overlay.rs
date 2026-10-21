use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicU64;
use std::sync::{RwLock, RwLockReadGuard};

use super::{content::FileContent, version::FileVersion};

#[derive(Clone, Debug)]
struct OverlayEntry {
    bytes: Vec<u8>,
    version: FileVersion,
}

/// In-memory overlay that shadows on-disk files for unsaved editor buffers.
#[derive(Debug, Default)]
pub struct Overlay {
    entries: RwLock<HashMap<PathBuf, OverlayEntry>>,
    tombstones: RwLock<HashMap<PathBuf, FileVersion>>,
    version_seed: AtomicU64,
}

impl Overlay {
    /// Creates an empty overlay.
    pub fn new() -> Self {
        Self::default()
    }

    fn read_lock(&self) -> RwLockReadGuard<'_, HashMap<PathBuf, OverlayEntry>> {
        self.entries.read().expect("overlay entries lock poisoned")
    }

    /// Returns overlay content when present.
    pub fn read(&self, path: &Path) -> Option<FileContent> {
        self.read_lock().get(path).map(|entry| FileContent { bytes: entry.bytes.clone(), version: entry.version })
    }

    /// Returns whether the path is hidden by a tombstone.
    pub fn is_removed(&self, path: &Path) -> bool {
        self.tombstones.read().expect("overlay tombstones lock poisoned").contains_key(path)
    }

    /// Inserts or replaces overlay content and returns the new version.
    pub fn write(&self, path: PathBuf, bytes: Vec<u8>) -> FileVersion {
        let version = FileVersion::next(&self.version_seed);
        self.tombstones.write().expect("overlay tombstones lock poisoned").remove(&path);
        self.entries.write().expect("overlay entries lock poisoned").insert(path, OverlayEntry { bytes, version });
        version
    }

    /// Removes overlay content and records a tombstone so layered reads skip disk.
    pub fn remove(&self, path: &Path) -> FileVersion {
        self.entries.write().expect("overlay entries lock poisoned").remove(path);
        let version = FileVersion::next(&self.version_seed);
        self.tombstones.write().expect("overlay tombstones lock poisoned").insert(path.to_path_buf(), version);
        version
    }

    /// Clears overlay state for `path`, restoring layered reads to the backing store.
    pub fn clear(&self, path: &Path) {
        self.entries.write().expect("overlay entries lock poisoned").remove(path);
        self.tombstones.write().expect("overlay tombstones lock poisoned").remove(path);
    }
}
