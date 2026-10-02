//! Domain provenance references used by diagnostic locations.

use core::fmt::{self, Display, Formatter};

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

impl Display for SemanticPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
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
