use std::path::{Path, PathBuf};

use super::{Entry, Error, Filter, Iter};

/// How a walk reacts to an error raised while visiting the tree.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ErrorPolicy {
    /// Yield the error and end the iteration, so the caller cannot overlook a partial walk.
    Strict,
    /// Yield the error and keep visiting the entries that are still pending.
    ///
    /// The error stays in the item stream. A caller that discards it silently accepts an incomplete
    /// tree, so use `Strict` when a partial result is not acceptable.
    #[default]
    Lenient,
}

/// Configuration for a recursive folder walk.
#[derive(Debug, Clone)]
pub struct Walker {
    pub(crate) root: PathBuf,
    pub(crate) min_depth: usize,
    pub(crate) max_depth: usize,
    pub(crate) follow_links: bool,
    pub(crate) sort_by_file_name: bool,
    pub(crate) error_policy: ErrorPolicy,
}

impl Walker {
    /// Create a walker rooted at `root`.
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            min_depth: 0,
            max_depth: usize::MAX,
            follow_links: false,
            sort_by_file_name: false,
            error_policy: ErrorPolicy::default(),
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
    ///
    /// A link that resolves to a folder already present on the current branch is reported as
    /// [`Error::loop_ancestor`] instead of being traversed again.
    pub fn follow_links(self, yes: bool) -> Self {
        Self { follow_links: yes, ..self }
    }

    /// Visit the entries of every folder in file-name order.
    ///
    /// Without this option the order inside a folder is the order the host reports, which is not
    /// stable across platforms or runs. Sorting makes a walk reproducible, at the cost of reading a
    /// whole folder before its first child is yielded.
    pub fn sort_by_file_name(self) -> Self {
        Self { sort_by_file_name: true, ..self }
    }

    /// Choose how an error affects the rest of the iteration.
    pub fn error_policy(self, policy: ErrorPolicy) -> Self {
        Self { error_policy: policy, ..self }
    }

    /// Apply `predicate` to every discovered entry.
    ///
    /// The predicate decides whether an entry is yielded and whether a folder is traversed, so
    /// rejecting a folder here prunes the whole subtree.
    pub fn filter<F>(self, predicate: F) -> Filtered<F>
    where
        F: FnMut(&Entry) -> Filter + 'static,
    {
        Filtered { walker: self, predicate }
    }
}

impl IntoIterator for Walker {
    type Item = Result<Entry, Error>;
    type IntoIter = Iter;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}

/// A [`Walker`] with a predicate that decides what is yielded and what is traversed.
pub struct Filtered<F> {
    walker: Walker,
    predicate: F,
}

impl<F> Filtered<F>
where
    F: FnMut(&Entry) -> Filter + 'static,
{
    /// Returns the configured root path.
    pub fn root(&self) -> &Path {
        self.walker.root()
    }
}

impl<F> IntoIterator for Filtered<F>
where
    F: FnMut(&Entry) -> Filter + 'static,
{
    type Item = Result<Entry, Error>;
    type IntoIter = Iter;

    fn into_iter(self) -> Self::IntoIter {
        Iter::filtered(self.walker, self.predicate)
    }
}
