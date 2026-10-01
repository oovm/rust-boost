//! External source lookup for text rendering.

use crate::{SourceCache, SourceID, SourcePath, SourceText};
use core::fmt::{self, Display, Formatter};
use std::io;

/// Failure to resolve source content from a provider.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderError {
    message: String,
}

impl ProviderError {
    fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

impl From<io::Error> for ProviderError {
    fn from(value: io::Error) -> Self {
        Self::new(value.to_string())
    }
}

impl Display for ProviderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ProviderError {}

/// Resolves source text and display metadata for renderers.
pub trait SourceProvider {
    /// Fetch source text for a cached identifier.
    fn fetch(&self, file: &SourceID) -> Result<&SourceText, ProviderError>;

    /// Returns the display path for a cached identifier.
    fn source_path(&self, file: &SourceID) -> Option<&SourcePath>;
}

impl SourceProvider for SourceCache {
    fn fetch(&self, file: &SourceID) -> Result<&SourceText, ProviderError> {
        SourceCache::fetch(self, file).map_err(ProviderError::from)
    }

    fn source_path(&self, file: &SourceID) -> Option<&SourcePath> {
        SourceCache::source_path(self, file)
    }
}

/// Stable source reference for cross-crate contracts.
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
