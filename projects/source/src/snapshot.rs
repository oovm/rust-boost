use core::fmt::{self, Display, Formatter};

use crate::identity::{IdentityError, SourceRef};

/// Immutable snapshot reference bound to a specific source revision.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SnapshotRef {
    source: SourceRef,
}

impl SnapshotRef {
    /// Create a snapshot reference from a source reference.
    pub fn new(source: SourceRef) -> Self {
        Self { source }
    }

    /// Create a snapshot reference with an explicit revision.
    pub fn with_revision(
        namespace: impl Into<String>,
        id: impl Into<String>,
        revision: impl Into<String>,
    ) -> Result<Self, IdentityError> {
        let source = SourceRef::new(namespace, id)?.with_revision(revision)?;
        Ok(Self { source })
    }

    /// Returns the underlying source reference.
    pub fn source(&self) -> &SourceRef {
        &self.source
    }
}

impl Display for SnapshotRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.source.fmt(f)
    }
}

/// Process-local snapshot handle allocated by a store.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SnapshotHandle(u32);

impl SnapshotHandle {
    /// Create a handle from a raw index.
    #[cfg(feature = "std")]
    pub(crate) fn from_index(index: u32) -> Self {
        Self(index)
    }

    /// Returns the raw store index.
    pub fn index(&self) -> u32 {
        self.0
    }
}
