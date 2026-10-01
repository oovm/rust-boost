#![doc = include_str!("readme.md")]
#![deny(missing_docs)]

mod error;
mod walker;

pub use error::WalkError;
pub use walker::{DirEntry, IntoIter, Walker};

#[cfg(feature = "async")]
mod async_walker;

#[cfg(feature = "async")]
pub use async_walker::{AsyncWalker, Filtering};

/// Compatibility alias for code migrating from `walkdir::WalkDir`.
pub type WalkDir = Walker;

#[cfg(feature = "async")]
/// Compatibility alias for code migrating from `async_walkdir::WalkDir`.
pub type AsyncWalkDir = AsyncWalker;
