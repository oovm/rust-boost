//! The explicit per-case context.
//!
//! A context is created by the runner for one execution and passed to a body that accepts it. It
//! carries the descriptor and the skip state, so a body never reaches global mutable state or a
//! thread-local guard. Capabilities that need a collector will extend this type instead of adding a
//! second ambient context.

use crate::descriptor::TestDescriptor;

/// The state owned by one test execution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestContext {
    descriptor: TestDescriptor,
    skip_reason: Option<String>,
}

impl TestContext {
    /// Creates a context for the given descriptor.
    pub fn new(descriptor: TestDescriptor) -> Self {
        Self { descriptor, skip_reason: None }
    }

    /// Returns the descriptor of the entry being executed.
    pub fn descriptor(&self) -> &TestDescriptor {
        &self.descriptor
    }

    /// Requests that the entry be reported as skipped.
    ///
    /// The body may continue after this call, but the runner records the skip and ignores the
    /// descriptor expectation. A later call replaces the reason.
    pub fn skip(&mut self, reason: impl Into<String>) {
        self.skip_reason = Some(reason.into());
    }

    /// Returns whether a skip was requested.
    pub fn is_skipped(&self) -> bool {
        self.skip_reason.is_some()
    }

    /// Returns the requested skip reason.
    pub fn skip_reason(&self) -> Option<&str> {
        self.skip_reason.as_deref()
    }
}
