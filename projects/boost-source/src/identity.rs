use core::fmt::{self, Display, Formatter};

use crate::range::ByteRange;

/// Logical source identity.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SourceIdentity {
    namespace: String,
    id: String,
}

impl SourceIdentity {
    /// Create a validated source identity.
    pub fn new(namespace: impl Into<String>, id: impl Into<String>) -> Result<Self, IdentityError> {
        let namespace = namespace.into();
        let id = id.into();
        if namespace.is_empty() || id.is_empty() {
            return Err(IdentityError::EmptyPart);
        }
        Ok(Self { namespace, id })
    }

    /// Returns the namespace.
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Returns the identifier.
    pub fn id(&self) -> &str {
        &self.id
    }
}

impl Display for SourceIdentity {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.namespace, self.id)
    }
}

/// Opaque content revision assigned by the snapshot owner.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SourceRevision(String);

impl SourceRevision {
    /// Create a revision identifier.
    pub fn new(revision: impl Into<String>) -> Result<Self, IdentityError> {
        let revision = revision.into();
        if revision.is_empty() {
            return Err(IdentityError::EmptyPart);
        }
        Ok(Self(revision))
    }

    /// Returns the revision string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Stable source reference with optional revision.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SourceRef {
    identity: SourceIdentity,
    revision: Option<SourceRevision>,
}

impl SourceRef {
    /// Create a source reference without revision.
    pub fn new(namespace: impl Into<String>, id: impl Into<String>) -> Result<Self, IdentityError> {
        Ok(Self { identity: SourceIdentity::new(namespace, id)?, revision: None })
    }

    /// Attach a revision or snapshot identifier.
    pub fn with_revision(mut self, revision: impl Into<String>) -> Result<Self, IdentityError> {
        self.revision = Some(SourceRevision::new(revision)?);
        Ok(self)
    }

    /// Returns the logical identity.
    pub fn identity(&self) -> &SourceIdentity {
        &self.identity
    }

    /// Returns the optional revision.
    pub fn revision(&self) -> Option<&SourceRevision> {
        self.revision.as_ref()
    }

    /// Returns the namespace.
    pub fn namespace(&self) -> &str {
        self.identity.namespace()
    }

    /// Returns the identifier.
    pub fn id(&self) -> &str {
        self.identity.id()
    }

    /// Bind the reference to a byte span inside the referenced snapshot.
    pub fn span(self, range: ByteRange) -> SourceSpan {
        SourceSpan { source: self, range }
    }

    /// Encode the reference into a reversible wire identifier.
    pub fn wire_id(&self) -> String {
        crate::wire::encode_source_ref(self)
    }
}

impl Display for SourceRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.revision.as_ref() {
            Some(revision) => write!(f, "{}@{}", self.identity, revision.as_str()),
            None => self.identity.fmt(f),
        }
    }
}

/// Display-only label that does not participate in identity comparison.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SourceLabel(String);

impl SourceLabel {
    /// Create a display label.
    pub fn new(label: impl Into<String>) -> Self {
        Self(label.into())
    }

    /// Returns the label text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for SourceLabel {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A source reference bound to a byte span.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SourceSpan {
    /// Referenced source.
    pub source: SourceRef,
    /// Half-open byte range.
    pub range: ByteRange,
}

/// Invalid identity or revision part.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdentityError {
    /// Namespace, id, or revision must not be empty.
    EmptyPart,
}

impl Display for IdentityError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPart => f.write_str("source identity parts must not be empty"),
        }
    }
}
