//! Structured outcomes and failures of one test execution.
//!
//! A result keeps the descriptor, the outcome, every failure in execution order, and the observed
//! duration. Failures are separate from the outcome: an expected failure is a successful result that
//! still carries the failure evidence that was observed.

use core::fmt::{self, Debug, Display, Formatter};
use std::{any::Any, time::Duration};

use crate::descriptor::{SourceLocation, TestDescriptor};

/// The phase of an execution that produced a failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Fixture or other setup before the body.
    Setup,
    /// The body itself.
    Body,
    /// Fixture or other teardown after the body.
    Teardown,
}

impl Phase {
    /// Returns the stable machine-readable name of the phase.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Setup => "setup",
            Self::Body => "body",
            Self::Teardown => "teardown",
        }
    }
}

impl Display for Phase {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// The assertion form that produced a failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssertionKind {
    /// A boolean condition was false.
    Condition,
    /// Two operands were expected to be equal.
    Equal,
    /// Two operands were expected to be different.
    NotEqual,
    /// An explicit failure was raised.
    Fail,
}

impl AssertionKind {
    /// Returns the stable machine-readable name of the assertion form.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Condition => "condition",
            Self::Equal => "equal",
            Self::NotEqual => "not-equal",
            Self::Fail => "fail",
        }
    }
}

impl Display for AssertionKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// The origin of a failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailureKind {
    /// A structured assertion failed.
    Assertion(AssertionKind),
    /// The body returned an error.
    Returned,
    /// The body panicked.
    Panic,
    /// The expected failure did not occur.
    Expectation,
}

impl FailureKind {
    /// Returns the stable machine-readable name of the failure kind.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Assertion(kind) => kind.as_str(),
            Self::Returned => "returned",
            Self::Panic => "panic",
            Self::Expectation => "expectation",
        }
    }
}

impl Display for FailureKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// One structured operand of a failure.
///
/// Plain scalar facts are kept as [`OperandValue::Text`] when the caller can express them without
/// `Debug`, while a generic asserted value keeps its `Debug` rendering under
/// [`OperandValue::Debug`]. Both are evidence, not the assertion identity itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OperandValue {
    /// A value the caller rendered without `Debug`.
    Text(String),
    /// A value rendered through `Debug`.
    Debug(String),
}

impl OperandValue {
    /// Returns the rendered text.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Text(value) | Self::Debug(value) => value,
        }
    }
}

/// A labeled operand attached to a failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Operand {
    label: String,
    value: OperandValue,
}

impl Operand {
    /// Creates an operand from text the caller has already rendered.
    pub fn text(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self { label: label.into(), value: OperandValue::Text(value.into()) }
    }

    /// Creates an operand by rendering a value through `Debug`.
    pub fn debug<T>(label: impl Into<String>, value: &T) -> Self
    where
        T: Debug + ?Sized,
    {
        Self { label: label.into(), value: OperandValue::Debug(format!("{value:?}")) }
    }

    /// Returns the operand label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Returns the structured value.
    pub fn value(&self) -> &OperandValue {
        &self.value
    }
}

impl Display for Operand {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.label, self.value.as_str())
    }
}

/// One structured failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestFailure {
    phase: Phase,
    kind: FailureKind,
    message: String,
    location: Option<SourceLocation>,
    operands: Vec<Operand>,
}

impl TestFailure {
    /// Creates a failure for a structured assertion.
    pub fn assertion(kind: AssertionKind, message: impl Into<String>, location: SourceLocation, operands: Vec<Operand>) -> Self {
        Self { phase: Phase::Body, kind: FailureKind::Assertion(kind), message: message.into(), location: Some(location), operands }
    }

    /// Creates a failure for a body that returned an error.
    pub fn returned(message: impl Into<String>) -> Self {
        Self { phase: Phase::Body, kind: FailureKind::Returned, message: message.into(), location: None, operands: Vec::new() }
    }

    /// Creates a failure for a panic whose payload was not a structured failure.
    pub fn panic(message: impl Into<String>) -> Self {
        Self { phase: Phase::Body, kind: FailureKind::Panic, message: message.into(), location: None, operands: Vec::new() }
    }

    /// Creates the failure reported when an expected failure did not happen.
    pub fn expectation(message: impl Into<String>) -> Self {
        Self { phase: Phase::Body, kind: FailureKind::Expectation, message: message.into(), location: None, operands: Vec::new() }
    }

    /// Replaces the phase of this failure.
    ///
    /// The runner assigns the phase when the failure is recorded; constructors describe the body.
    pub fn with_phase(mut self, phase: Phase) -> Self {
        self.phase = phase;
        self
    }

    /// Returns the phase that produced the failure.
    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// Returns the failure kind.
    pub fn kind(&self) -> FailureKind {
        self.kind
    }

    /// Returns the failure message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the captured source location, when the failure came from a located assertion.
    pub fn location(&self) -> Option<&SourceLocation> {
        self.location.as_ref()
    }

    /// Returns the structured operands in declaration order.
    pub fn operands(&self) -> &[Operand] {
        &self.operands
    }
}

impl Display for TestFailure {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for TestFailure {}

/// The category of a completed execution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TestOutcome {
    /// The body completed normally and no skip was requested.
    Passed,
    /// The body failed.
    Failed,
    /// The body asked the harness to skip it.
    Skipped,
    /// The body failed as declared by the descriptor.
    ExpectedFailure,
}

impl TestOutcome {
    /// Returns whether this outcome is not a failure.
    ///
    /// A skipped entry and an observed expected failure both keep the standard harness green.
    pub const fn is_success(self) -> bool {
        !matches!(self, Self::Failed)
    }

    /// Returns the stable machine-readable name of the outcome.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
            Self::ExpectedFailure => "expected-failure",
        }
    }
}

impl Display for TestOutcome {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// The structured result of one test execution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestResult {
    descriptor: TestDescriptor,
    outcome: TestOutcome,
    failures: Vec<TestFailure>,
    duration: Duration,
    skip_reason: Option<String>,
}

impl TestResult {
    /// Creates a result from the facts observed by the runner.
    pub fn new(
        descriptor: TestDescriptor,
        outcome: TestOutcome,
        failures: Vec<TestFailure>,
        duration: Duration,
        skip_reason: Option<String>,
    ) -> Self {
        Self { descriptor, outcome, failures, duration, skip_reason }
    }

    /// Returns the descriptor of the executed entry.
    pub fn descriptor(&self) -> &TestDescriptor {
        &self.descriptor
    }

    /// Returns the outcome category.
    pub fn outcome(&self) -> TestOutcome {
        self.outcome
    }

    /// Returns every failure in execution order.
    pub fn failures(&self) -> &[TestFailure] {
        &self.failures
    }

    /// Returns the observed duration.
    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// Returns the reason supplied through [`TestContext::skip`](crate::TestContext::skip).
    pub fn skip_reason(&self) -> Option<&str> {
        self.skip_reason.as_deref()
    }

    /// Returns whether the result keeps the standard harness green.
    pub fn is_success(&self) -> bool {
        self.outcome.is_success()
    }

    /// Renders the stable text projection of this result.
    pub fn render_text(&self) -> String {
        crate::report::render_text(self)
    }
}

/// Maps an error returned by a test body into a structured failure.
///
/// The blanket implementation covers every `Display + 'static` error. A [`TestFailure`] value keeps
/// its structured phase, location, and operands instead of being flattened into its message, so a
/// body may return either its own error type or a failure built by an assertion helper.
pub trait IntoTestFailure {
    /// Consumes the error and returns the failure evidence.
    fn into_test_failure(self) -> TestFailure;
}

impl<E> IntoTestFailure for E
where
    E: Display + 'static,
{
    fn into_test_failure(self) -> TestFailure {
        let value: Box<dyn Any> = Box::new(self);
        match value.downcast::<TestFailure>() {
            Ok(failure) => *failure,
            Err(value) => {
                let message = value.downcast::<E>().expect("the boxed value keeps its original type").to_string();
                TestFailure::returned(message)
            }
        }
    }
}
