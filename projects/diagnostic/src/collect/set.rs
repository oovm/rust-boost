use alloc::vec::Vec;

use crate::model::Diagnostic;

use super::SinkStatus;

/// Ordered diagnostic collection with truncation metadata.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DiagnosticSet {
    diagnostics: Vec<Diagnostic>,
    truncated: bool,
    dropped: u64,
    max_diagnostics: Option<usize>,
}

impl DiagnosticSet {
    /// Create an empty diagnostic set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Limit the number of stored diagnostics.
    pub fn with_max_diagnostics(mut self, max: usize) -> Self {
        self.max_diagnostics = Some(max);
        self
    }

    /// Push a diagnostic, preserving insertion order.
    pub fn push(&mut self, diagnostic: Diagnostic) -> SinkStatus {
        if let Some(max) = self.max_diagnostics {
            if self.diagnostics.len() >= max {
                self.truncated = true;
                self.dropped = self.dropped.saturating_add(1);
                return SinkStatus::Truncated;
            }
        }
        self.diagnostics.push(diagnostic);
        SinkStatus::Accepted
    }

    /// Merge another set after this one, preserving order and truncation metadata.
    pub fn extend(&mut self, other: DiagnosticSet) {
        for diagnostic in other.diagnostics {
            let _ = self.push(diagnostic);
        }
        if other.truncated {
            self.truncated = true;
            self.dropped = self.dropped.saturating_add(other.dropped);
        }
    }

    /// Returns stored diagnostics in insertion order.
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Returns whether the set overflowed its limit.
    pub fn truncated(&self) -> bool {
        self.truncated
    }

    /// Returns the number of dropped diagnostics after truncation.
    pub fn dropped(&self) -> u64 {
        self.dropped
    }

    /// Returns diagnostics sorted by stable presentation key.
    pub fn sorted_by_stable_key(&self) -> Vec<&Diagnostic> {
        let mut items: Vec<_> = self.diagnostics.iter().collect();
        items.sort_by(stable_key);
        items
    }
}

fn stable_key(left: &&Diagnostic, right: &&Diagnostic) -> core::cmp::Ordering {
    left.origin()
        .cmp(right.origin())
        .then_with(|| left.code().cmp(right.code()))
        .then_with(|| left.severity().cmp(&right.severity()))
        .then_with(|| left.message().key().cmp(right.message().key()))
}
