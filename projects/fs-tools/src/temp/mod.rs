//! Temporary files and folders.

mod env;
mod file;
mod folder;
mod unique;

pub use env::{configure_folder, folder_path, set_folder};
pub use file::{create_file, create_file_in, create_named, create_named_in, NamedFile};
pub use folder::{create_folder, create_folder_in, TempFolder};
