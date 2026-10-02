use std::collections::BTreeMap;

use source_cache::SourceID;

use crate::SourceRef;

/// Maps stable [`SourceRef`] wire identifiers to cached [`SourceID`] values.
#[derive(Clone, Debug, Default)]
pub struct SourceRegistry {
    entries: BTreeMap<String, SourceID>,
}

impl SourceRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a source reference.
    pub fn register(&mut self, source: &SourceRef, id: SourceID) -> &mut Self {
        self.entries.insert(source.wire_id(), id);
        self
    }

    /// Resolve a source reference to a cached identifier.
    pub fn resolve(&self, source: &SourceRef) -> Option<&SourceID> {
        self.entries.get(&source.wire_id())
    }
}
