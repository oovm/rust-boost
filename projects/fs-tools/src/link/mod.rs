//! Links and path canonicalization.

pub use std::fs::{canonicalize, hard_link, read_link};

#[cfg(unix)]
pub use std::os::unix::fs::symlink;
