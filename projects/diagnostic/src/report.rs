//! Domain result paired with a diagnostic collection.

use crate::set::DiagnosticSet;

/// Domain result paired with a unified diagnostic collection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report<T, E> {
    /// Domain-specific success or failure value.
    pub result: Result<T, E>,
    /// Unified diagnostics emitted while producing the result.
    pub diagnostics: DiagnosticSet,
}

impl<T, E> Report<T, E> {
    /// Create a report from a result and diagnostic set.
    pub fn new(result: Result<T, E>, diagnostics: DiagnosticSet) -> Self {
        Self { result, diagnostics }
    }
}
