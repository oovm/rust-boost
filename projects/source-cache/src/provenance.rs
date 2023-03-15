//! Stable source identity and provenance primitives shared across renderers and diagnostics.

use core::fmt::{self, Display, Formatter};

/// Half-open byte range `[start, end)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ByteRange {
    /// Inclusive start offset.
    pub start: u64,
    /// Exclusive end offset.
    pub end: u64,
}

impl ByteRange {
    /// Create a byte range and validate ordering.
    pub fn new(start: u64, end: u64) -> Result<Self, RangeError> {
        if start > end {
            return Err(RangeError { start, end });
        }
        Ok(Self { start, end })
    }

    /// Returns whether the range is empty.
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// Invalid byte range ordering.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RangeError {
    start: u64,
    end: u64,
}

impl Display for RangeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "byte range start {} is after end {}", self.start, self.end)
    }
}

/// Namespace-qualified source reference with optional revision.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SourceRef {
    namespace: String,
    id: String,
    revision: Option<String>,
}

impl SourceRef {
    /// Create a stable source reference.
    pub fn new(namespace: impl Into<String>, id: impl Into<String>) -> Self {
        Self { namespace: namespace.into(), id: id.into(), revision: None }
    }

    /// Attach a revision or snapshot identifier.
    pub fn with_revision(mut self, revision: impl Into<String>) -> Self {
        self.revision = Some(revision.into());
        self
    }

    /// Returns the namespace.
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Returns the identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the optional revision.
    pub fn revision(&self) -> Option<&str> {
        self.revision.as_deref()
    }

    /// Render a stable wire identifier.
    pub fn to_wire_id(&self) -> String {
        match &self.revision {
            Some(revision) => format!("{}:{}@{}", self.namespace, self.id, revision),
            None => format!("{}:{}", self.namespace, self.id),
        }
    }
}

/// Address space reference for binary locations.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AddressSpaceRef {
    namespace: String,
    id: String,
}

impl AddressSpaceRef {
    /// Create an address space reference.
    pub fn new(namespace: impl Into<String>, id: impl Into<String>) -> Self {
        Self { namespace: namespace.into(), id: id.into() }
    }

    /// Returns the namespace.
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Returns the address space identifier.
    pub fn id(&self) -> &str {
        &self.id
    }
}

/// One segment in a container member path.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MemberSegment {
    kind: String,
    name: String,
}

impl MemberSegment {
    /// Create a member path segment.
    pub fn new(kind: impl Into<String>, name: impl Into<String>) -> Self {
        Self { kind: kind.into(), name: name.into() }
    }

    /// Returns the segment kind.
    pub fn kind(&self) -> &str {
        &self.kind
    }

    /// Returns the segment name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Path to a member inside a container source.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MemberPath {
    segments: Vec<MemberSegment>,
}

impl MemberPath {
    /// Create a member path from segments.
    pub fn new(segments: Vec<MemberSegment>) -> Self {
        Self { segments }
    }

    /// Returns the path segments.
    pub fn segments(&self) -> &[MemberSegment] {
        &self.segments
    }
}

/// Object reference inside a structured source.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ObjectRef {
    kind: String,
    id: String,
}

impl ObjectRef {
    /// Create an object reference.
    pub fn new(kind: impl Into<String>, id: impl Into<String>) -> Self {
        Self { kind: kind.into(), id: id.into() }
    }

    /// Returns the object kind.
    pub fn kind(&self) -> &str {
        &self.kind
    }

    /// Returns the object identifier.
    pub fn id(&self) -> &str {
        &self.id
    }
}

/// Document reference for semantic locations.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DocumentRef {
    namespace: String,
    id: String,
}

impl DocumentRef {
    /// Create a document reference.
    pub fn new(namespace: impl Into<String>, id: impl Into<String>) -> Self {
        Self { namespace: namespace.into(), id: id.into() }
    }

    /// Returns the namespace.
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Returns the document identifier.
    pub fn id(&self) -> &str {
        &self.id
    }
}

/// Semantic path inside a document.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SemanticPath(String);

impl SemanticPath {
    /// Create a semantic path.
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    /// Returns the path string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Mapping precision for container provenance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum MappingPrecision {
    /// Exact byte mapping is known.
    Exact,
    /// Mapping is known only at container granularity.
    Container,
    /// Mapping is unknown.
    Unknown,
}
