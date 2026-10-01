//! Temporary folder configuration.

use std::{
    io,
    path::{Path, PathBuf},
    sync::RwLock,
};

static OVERRIDE: RwLock<Option<PathBuf>> = RwLock::new(None);

/// Returns the folder used for temporary files and folders.
///
/// When [`set_folder`] was called, that path is returned. Otherwise this falls back to
/// [`std::env::temp_dir`] on hosts that define one.
///
/// On WASI there is no platform default. Call [`set_folder`] or [`configure_folder`] first,
/// or pass an explicit base to [`super::create_folder_in`](super::create_folder_in).
pub fn folder_path() -> io::Result<PathBuf> {
    if let Some(path) = OVERRIDE.read().expect("temp override lock poisoned").clone() {
        return Ok(path);
    }

    #[cfg(target_os = "wasi")]
    {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "temporary folder is not configured. call temp::configure_folder or temp::create_folder_in",
        ));
    }

    #[cfg(not(target_os = "wasi"))]
    {
        Ok(std::env::temp_dir())
    }
}

/// Override the temporary folder for the remainder of this process.
///
/// On WASI this must be called with a preopened writable folder before creating
/// temporary paths. Android apps may need to point this at the per-app cache folder.
pub fn set_folder(path: impl AsRef<Path>) {
    *OVERRIDE.write().expect("temp override lock poisoned") = Some(path.as_ref().to_path_buf());
}

/// Create `path` when missing and register it as the process temporary folder.
pub fn configure_folder(path: impl AsRef<Path>) -> io::Result<()> {
    let path = path.as_ref();
    crate::create_folder_all(path)?;
    set_folder(path);
    Ok(())
}
