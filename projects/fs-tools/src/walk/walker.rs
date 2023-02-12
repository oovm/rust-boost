use std::path::{Path, PathBuf};

use super::{Error, Entry, Iter};

/// Configuration for a recursive folder walk.
#[derive(Debug, Clone)]
pub struct Walker {
    pub(crate) root: PathBuf,
    pub(crate) min_depth: usize,
    pub(crate) max_depth: usize,
    pub(crate) follow_links: bool,
}

impl Walker {
    /// Create a walker rooted at `root`.
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            min_depth: 0,
            max_depth: usize::MAX,
            follow_links: false,
        }
    }

    /// Returns the configured root path.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Set the minimum depth of entries yielded by the iterator.
    pub fn min_depth(self, depth: usize) -> Self {
        Self { min_depth: depth, ..self }
    }

    /// Set the maximum depth of entries yielded by the iterator.
    pub fn max_depth(self, depth: usize) -> Self {
        Self { max_depth: depth, ..self }
    }

    /// Follow symbolic links while walking directories.
    pub fn follow_links(self, yes: bool) -> Self {
        Self { follow_links: yes, ..self }
    }
}

impl IntoIterator for Walker {
    type Item = Result<Entry, Error>;
    type IntoIter = Iter;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}
