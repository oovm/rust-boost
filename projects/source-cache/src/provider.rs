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
