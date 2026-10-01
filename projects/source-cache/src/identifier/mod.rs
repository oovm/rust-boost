use crate::SourceSpan;
use std::{
    borrow::Cow,
    fmt::{Debug, Display, Formatter},
    hash::{DefaultHasher, Hash, Hasher},
    ops::Range,
    path::PathBuf,
};
use url::Url;

mod display;

/// The source path
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SourcePath {
    /// This is an anonymous identifier
    #[default]
    Anonymous,
    /// This is a snippet of identifier
    Snippet(Cow<'static, str>),
    /// This is a local identifier
    Local(PathBuf),
    /// This is a remote identifier
    Remote(Url),
}

/// A type representing a single line of a [`Source`].
#[derive(Copy, Clone, Default, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SourceID {
    pub(crate) hash: u64,
}

impl SourcePath {
    /// Calculate the file from the identifier
    pub fn source_id(&self) -> SourceID {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        SourceID { hash: hasher.finish() }
    }
}

impl SourceID {
    /// Create a [`SourceID`] from a path description.
    pub fn from_path(path: &SourcePath) -> Self {
        path.source_id()
    }

    /// Create a new [`SourceID`] with the given ID.
    pub fn with_range(self, range: Range<u32>) -> SourceSpan {
        SourceSpan { start: range.start, end: range.end, file: self }
    }
}
