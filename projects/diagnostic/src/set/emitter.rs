use crate::model::Diagnostic;

use super::DiagnosticSet;

/// Result of pushing a diagnostic into a collector.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmitStatus {
    /// Diagnostic was accepted.
    Accepted,
    /// Diagnostic was rejected because the collection is full.
    Truncated,
}

/// Append-only diagnostic collector.
#[derive(Debug)]
pub struct DiagnosticEmitter<'a> {
    target: &'a mut DiagnosticSet,
}

impl<'a> DiagnosticEmitter<'a> {
    /// Borrow a collector over the given set.
    pub fn new(target: &'a mut DiagnosticSet) -> Self {
        Self { target }
    }

    /// Push one diagnostic into the target set.
    pub fn emit(&mut self, diagnostic: Diagnostic) -> EmitStatus {
        self.target.push(diagnostic)
    }
}
