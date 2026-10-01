#![doc = include_str!("readme.md")]
#![deny(missing_docs)]

//! Extended filesystem API modeled after [`std::fs`].
//!
//! Standard operations are re-exported at the crate root. Recursive traversal lives in
//! [`walk`] and [`async_walk`].

pub mod folder;
pub mod file;
pub mod link;
pub mod metadata;
pub mod permissions;
pub mod read_write;
pub mod remove;
pub mod rename;
pub mod temp;
pub mod walk;

#[cfg(feature = "async")]
pub mod async_walk;

pub use folder::{create_folder, create_folder_all, read_dir, FolderEntry, ReadFolder};
pub use file::{exists, File, OpenOptions};
pub use link::{canonicalize, hard_link, read_link};
#[cfg(unix)]
pub use link::symlink;
pub use metadata::{metadata, symlink_metadata, FileType, Metadata};
pub use permissions::{set_permissions, Permissions};
pub use read_write::{copy, read, read_to_string, write};
pub use remove::{remove_dir, remove_dir_all, remove_file};
pub use rename::rename;
