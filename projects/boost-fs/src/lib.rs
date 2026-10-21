#![doc = include_str!("readme.md")]
#![deny(missing_docs)]

//! Extended filesystem API modeled after [`std::fs`].
//!
//! Standard operations are re-exported at the crate root. Recursive traversal lives in
//! [`walk`] and [`async_walk`].

pub mod file;
pub mod folder;
pub mod link;
pub mod metadata;
pub mod permissions;
pub mod publish;
pub mod read_write;
pub mod remove;
pub mod rename;
pub mod temp;
pub mod vfs;
pub mod walk;
pub mod watch;

#[cfg(feature = "async")]
pub mod async_walk;

pub use file::{File, OpenOptions, exists};
pub use folder::{FolderEntry, ReadFolder, create_folder, create_folder_all, read_dir};
#[cfg(unix)]
pub use link::symlink;
pub use link::{canonicalize, hard_link, read_link};
pub use metadata::{FileType, Metadata, metadata, symlink_metadata};
pub use permissions::{Permissions, set_permissions};
pub use publish::{AtomicPublisher, OverwritePolicy, PublishOptions, publish_bytes, publish_bytes_with_options};
pub use read_write::{copy, read, read_to_string, write};
pub use remove::{remove_file, remove_folder, remove_folder_all};
pub use rename::rename;
