//! Snapshot scanning used by the polling backend.

use std::{
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
    time::SystemTime,
};

use super::{RescanReason, WatchError, event::WatchEvent};

/// State compared between two polling samples.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Fingerprint {
    is_dir: bool,
    len: u64,
    modified: Option<SystemTime>,
    readonly: bool,
    #[cfg(unix)]
    mode: u32,
    #[cfg(windows)]
    attributes: u32,
}

impl Fingerprint {
    fn from_metadata(metadata: &fs::Metadata) -> Self {
        Self {
            is_dir: metadata.is_dir(),
            len: metadata.len(),
            modified: metadata.modified().ok(),
            readonly: metadata.permissions().readonly(),
            #[cfg(unix)]
            mode: {
                use std::os::unix::fs::MetadataExt;
                metadata.mode()
            },
            #[cfg(windows)]
            attributes: {
                use std::os::windows::fs::MetadataExt;
                metadata.file_attributes()
            },
        }
    }
}

/// A path and the failure that stopped a scan.
#[derive(Debug)]
pub(super) struct ScanError {
    pub(super) path: PathBuf,
    pub(super) error: io::Error,
}

impl ScanError {
    /// Whether the watched root itself disappeared.
    pub(super) fn is_missing_root(&self, root: &Path) -> bool {
        self.path == root && self.error.kind() == io::ErrorKind::NotFound
    }

    /// Convert the scan failure to the public error model.
    pub(super) fn into_watch_error(self) -> WatchError {
        super::error::path_error(&self.path, self.error)
    }
}

/// Resolve a subscription root to an absolute, symlink-free path.
///
/// Verbatim prefixes added by [`fs::canonicalize`] on Windows are removed so delivered paths keep
/// the spelling consumers expect.
pub(super) fn canonical_root(path: &Path) -> Result<PathBuf, WatchError> {
    let resolved = fs::canonicalize(path).map_err(|error| super::error::path_error(path, error))?;
    let resolved = normalize(resolved);
    let metadata = fs::metadata(&resolved).map_err(|error| super::error::path_error(path, error))?;
    if !metadata.is_dir() {
        return Err(WatchError::NotDirectory(path.to_path_buf()));
    }
    Ok(resolved)
}

/// Resolve a path for subscription lookup without requiring it to exist.
pub(super) fn subscription_key(path: &Path) -> PathBuf {
    fs::canonicalize(path).map(normalize).unwrap_or_else(|_| std::path::absolute(path).map(normalize).unwrap_or_else(|_| path.to_path_buf()))
}

#[cfg(windows)]
fn normalize(path: PathBuf) -> PathBuf {
    use std::path::Prefix;

    let mut components = path.components();
    let Some(std::path::Component::Prefix(prefix)) = components.next()
    else {
        return path;
    };
    let rest: PathBuf = components.collect();
    match prefix.kind() {
        Prefix::VerbatimDisk(drive) => {
            let mut normalized = PathBuf::from(format!("{}:\\", drive as char));
            normalized.push(rest);
            normalized
        }
        Prefix::VerbatimUNC(server, share) => {
            let mut normalized = PathBuf::from(format!(r"\\{}\{}", server.to_string_lossy(), share.to_string_lossy()));
            normalized.push(rest);
            normalized
        }
        _ => path,
    }
}

#[cfg(not(windows))]
fn normalize(path: PathBuf) -> PathBuf {
    path
}

/// Snapshot every entry below `root` without following symlinked folders.
pub(super) fn scan_tree(root: &Path) -> Result<BTreeMap<PathBuf, Fingerprint>, ScanError> {
    let metadata = fs::metadata(root).map_err(|error| ScanError { path: root.to_path_buf(), error })?;
    if !metadata.is_dir() {
        return Err(ScanError { path: root.to_path_buf(), error: io::Error::from(io::ErrorKind::NotADirectory) });
    }

    let mut snapshot = BTreeMap::new();
    let mut folders = vec![root.to_path_buf()];
    while let Some(folder) = folders.pop() {
        let entries = match fs::read_dir(&folder) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(ScanError { path: folder, error }),
        };
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                Err(error) => return Err(ScanError { path: folder.clone(), error }),
            };
            let path = entry.path();
            let file_type = match entry.file_type() {
                Ok(file_type) => file_type,
                Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                Err(error) => return Err(ScanError { path: path.clone(), error }),
            };
            let metadata = match entry.metadata() {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                Err(error) => return Err(ScanError { path: path.clone(), error }),
            };
            if file_type.is_dir() {
                folders.push(path.clone());
            }
            snapshot.insert(path, Fingerprint::from_metadata(&metadata));
        }
    }
    Ok(snapshot)
}

/// Turn two snapshots into events.
pub(super) fn diff(previous: &BTreeMap<PathBuf, Fingerprint>, current: &BTreeMap<PathBuf, Fingerprint>) -> Vec<WatchEvent> {
    let mut events = Vec::new();
    for path in previous.keys() {
        if !current.contains_key(path) {
            events.push(WatchEvent::Removed(path.clone()));
        }
    }
    for (path, fingerprint) in current {
        match previous.get(path) {
            None => {
                events.push(WatchEvent::Created(path.clone()));
                if fingerprint.is_dir {
                    events.push(WatchEvent::Rescan { root: path.clone(), reason: RescanReason::NewDirectory });
                }
            }
            Some(previous) if previous != fingerprint => events.push(WatchEvent::Modified(path.clone())),
            Some(_) => {}
        }
    }
    events
}
