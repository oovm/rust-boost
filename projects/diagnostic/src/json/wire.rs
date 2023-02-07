use crate::{Diagnostic, DiagnosticSet};

/// Current JSON schema version for diagnostic wire data.
pub const SCHEMA_VERSION: u32 = 1;

/// Diagnostic collection envelope for JSON transport.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticEnvelope {
    /// Wire schema version.
    pub schema_version: u32,
    /// Ordered diagnostics.
    pub diagnostics: Vec<Diagnostic>,
    /// Whether the producer truncated the collection.
    pub truncated: bool,
    /// Number of dropped diagnostics after truncation.
    pub dropped: u64,
}

impl DiagnosticEnvelope {
    /// Serialize a diagnostic set into the wire envelope.
    pub fn from_set(set: &DiagnosticSet) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            diagnostics: set.diagnostics().to_vec(),
            truncated: set.truncated(),
            dropped: set.dropped(),
        }
    }
}
