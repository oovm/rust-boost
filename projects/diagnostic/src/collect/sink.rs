use super::DiagnosticSet;

use crate::model::Diagnostic;

/// Result of pushing a diagnostic into a sink-backed collection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SinkStatus {
    /// Diagnostic was accepted.
    Accepted,
    /// Diagnostic was rejected because the collection is full.
    Truncated,
}

/// Append-only diagnostic sink over a caller-owned set.
#[derive(Debug)]
pub struct DiagnosticSink<'a> {
    target: &'a mut DiagnosticSet,
}

impl<'a> DiagnosticSink<'a> {
    /// Borrow a sink over the given set.
    pub fn new(target: &'a mut DiagnosticSet) -> Self {
        Self { target }
    }

    /// Push one diagnostic into the target set.
    pub fn push(&mut self, diagnostic: Diagnostic) -> SinkStatus {
        self.target.push(diagnostic)
    }
}
