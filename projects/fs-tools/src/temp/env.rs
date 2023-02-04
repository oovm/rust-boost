//! Temporary directory configuration.

use std::{
    io,
    path::{Path, PathBuf},
    sync::RwLock,
};

static OVERRIDE: RwLock<Option<PathBuf>> = RwLock::new(None);

/// Returns the directory used for temporary files and directories.
///
/// When [`set_dir`] was called, that path is returned. Otherwise this falls back to
/// [`std::env::temp_dir`] on hosts that define one.
///
/// On WASI there is no platform default. Call [`set_dir`] or [`configure_dir`] first,
/// or pass an explicit base to [`super::create_dir_in`](super::create_dir_in).
pub fn dir_path() -> io::Result<PathBuf> {
    if let Some(path) = OVERRIDE.read().expect("temp override lock poisoned").clone() {
        return Ok(path);
    }

    #[cfg(target_os = "wasi")]
    {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "temporary directory is not configured. call temp::configure_dir or temp::create_dir_in",
        ));
    }

    #[cfg(not(target_os = "wasi"))]
    {
        Ok(std::env::temp_dir())
    }
}

/// Override the temporary directory for the remainder of this process.
///
/// On WASI this must be called with a preopened writable directory before creating
/// temporary paths. Android apps may need to point this at the per-app cache directory.
pub fn set_dir(path: impl AsRef<Path>) {
    *OVERRIDE.write().expect("temp override lock poisoned") = Some(path.as_ref().to_path_buf());
}

/// Create `path` when missing and register it as the process temporary directory.
pub fn configure_dir(path: impl AsRef<Path>) -> io::Result<()> {
    let path = path.as_ref();
    crate::create_dir_all(path)?;
    set_dir(path);
    Ok(())
}
