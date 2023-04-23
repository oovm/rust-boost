//! Temporary files and folders.

mod env;
mod file;
mod folder;
mod unique;

pub use env::{configure_folder, folder_path, set_folder};
pub use file::{NamedFile, create_file, create_file_in, create_named, create_named_in};
pub use folder::{TempFolder, create_folder, create_folder_in};

/// Number of candidate names tried before a creation is reported as a collision.
///
/// A candidate comes from the host entropy source, so a collision is possible but rare. Reaching the
/// limit therefore points at a failing entropy source or at a folder whose earlier temporary paths
/// are not being cleaned up, and reporting it is better than looping forever.
pub(crate) const RETRY_LIMIT: usize = 128;

/// Mode of a temporary file on a host that carries a mode.
///
/// A temporary file holds bytes its owner has not published yet, so it is created readable and
/// writable by the owner only instead of being narrowed after the fact.
#[cfg(unix)]
pub(crate) const OWNER_ONLY_FILE: u32 = 0o600;

/// Mode of a temporary folder on a host that carries a mode.
#[cfg(unix)]
pub(crate) const OWNER_ONLY_FOLDER: u32 = 0o700;
