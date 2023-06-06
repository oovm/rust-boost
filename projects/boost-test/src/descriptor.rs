//! Stable identity and declarative facts of a test entry.
//!
//! A descriptor is created before a body runs and never changes during one execution. The identity
//! is the module path plus the declared name, not an address, a thread id, or a run counter, so the
//! same entry keeps the same name across processes.

use core::fmt::{self, Display, Formatter};

use crate::case::CaseId;

/// A capability that a descriptor advertises.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Capability {
    /// A correctness test entry.
    Test,
    /// A benchmark measurement entry.
    Benchmark,
    /// A source probe that contributes to coverage.
    Coverage,
    /// An event trace entry.
    Trace,
}

impl Capability {
    /// Returns the stable machine-readable name of the capability.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Test => "test",
            Self::Benchmark => "benchmark",
            Self::Coverage => "coverage",
            Self::Trace => "trace",
        }
    }
}

impl Display for Capability {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A source position captured by a macro or supplied by a caller.
///
/// The location is evidence for a report, not a runtime identity: moving a function changes the
/// file and line, while the descriptor name stays stable.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SourceLocation {
    file: String,
    line: u32,
    column: u32,
}

impl SourceLocation {
    /// Creates a location from a file name and a one-based line and column.
    pub fn new(file: impl Into<String>, line: u32, column: u32) -> Self {
        Self { file: file.into(), line, column }
    }

    /// Returns the file name.
    pub fn file(&self) -> &str {
        &self.file
    }

    /// Returns the one-based line number.
    pub fn line(&self) -> u32 {
        self.line
    }

    /// Returns the one-based column number.
    pub fn column(&self) -> u32 {
        self.column
    }
}

impl Display for SourceLocation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}:{}", self.file, self.line, self.column)
    }
}

/// The outcome that the body is expected to produce.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Expectation {
    /// The body is expected to complete normally.
    #[default]
    Pass,
    /// The body is expected to fail.
    ///
    /// A failure is reported as [`TestOutcome::ExpectedFailure`](crate::TestOutcome::ExpectedFailure),
    /// while a normal completion is itself a failure.
    ExpectedFailure,
}

impl Expectation {
    /// Returns the stable machine-readable name of the expectation.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::ExpectedFailure => "expected-failure",
        }
    }
}

impl Display for Expectation {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Identity and declarative facts of one test entry.
///
/// The descriptor is owned by the context and cloned into the result, so a report can identify the
/// entry after the body has finished.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestDescriptor {
    name: String,
    module: String,
    source: SourceLocation,
    case: Option<CaseId>,
    expectation: Expectation,
    capabilities: Vec<Capability>,
}

impl TestDescriptor {
    /// Creates a descriptor for a test entry.
    ///
    /// The initial capability set contains [`Capability::Test`] only. Use
    /// [`with_capability`](Self::with_capability) to advertise additional capabilities.
    pub fn new(name: impl Into<String>, module: impl Into<String>, source: SourceLocation) -> Self {
        Self {
            name: name.into(),
            module: module.into(),
            source,
            case: None,
            expectation: Expectation::Pass,
            capabilities: vec![Capability::Test],
        }
    }

    /// Returns the declared name, without the module path.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the module path captured by the caller.
    pub fn module(&self) -> &str {
        &self.module
    }

    /// Returns the source location captured by the caller.
    pub fn source(&self) -> &SourceLocation {
        &self.source
    }

    /// Returns the stable case identity, when the entry belongs to a parameter table.
    pub fn case(&self) -> Option<&CaseId> {
        self.case.as_ref()
    }

    /// Returns the expected outcome.
    pub fn expectation(&self) -> Expectation {
        self.expectation
    }

    /// Returns the advertised capabilities in stable order.
    pub fn capabilities(&self) -> &[Capability] {
        &self.capabilities
    }

    /// Returns the stable qualified name, which is `module::name`, `name` for an empty module, or
    /// `module::name[case]` when a case identity is attached.
    pub fn qualified_name(&self) -> String {
        let base = if self.module.is_empty() { self.name.clone() } else { format!("{}::{}", self.module, self.name) };
        match &self.case {
            Some(case) => format!("{base}[{case}]"),
            None => base,
        }
    }

    /// Attaches a stable case identity.
    pub fn with_case(mut self, case: CaseId) -> Self {
        self.case = Some(case);
        self
    }

    /// Replaces the expected outcome.
    pub fn with_expectation(mut self, expectation: Expectation) -> Self {
        self.expectation = expectation;
        self
    }

    /// Adds a capability if it is not already advertised.
    pub fn with_capability(mut self, capability: Capability) -> Self {
        if !self.capabilities.contains(&capability) {
            self.capabilities.push(capability);
            self.capabilities.sort_unstable();
        }
        self
    }
}

impl Display for TestDescriptor {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.qualified_name())
    }
}
