//! Synchronous execution of one test body.
//!
//! The runner owns the phase order: it creates the context, catches a structured assertion payload
//! or an ordinary panic, applies the descriptor expectation, and returns the result. The result is
//! built even when the body panics, because the failure evidence must survive the unwind.

use std::{
    any::Any,
    panic::{AssertUnwindSafe, catch_unwind},
    time::Instant,
};

use crate::{
    context::TestContext,
    descriptor::{Expectation, TestDescriptor},
    result::{IntoTestFailure, TestFailure, TestOutcome, TestResult},
};

/// Executes `body` and returns the structured result.
///
/// A body that returns an error is reported as a failure through [`IntoTestFailure`]. A body that
/// panics is caught and reported as a failure too: a [`TestFailure`] payload keeps its location and
/// operands, while any other payload is reduced to its message. A skip requested through the context
/// takes precedence over the descriptor expectation, so an expected failure can still be skipped.
pub fn execute<F, E>(descriptor: TestDescriptor, body: F) -> TestResult
where
    F: FnOnce(&mut TestContext) -> Result<(), E>,
    E: IntoTestFailure,
{
    let expectation = descriptor.expectation();
    let qualified_name = descriptor.qualified_name();
    let started = Instant::now();
    let mut context = TestContext::new(descriptor.clone());

    let execution = catch_unwind(AssertUnwindSafe(|| body(&mut context)));
    let (mut outcome, mut failures) = match execution {
        Ok(Ok(())) => {
            if context.is_skipped() {
                (TestOutcome::Skipped, Vec::new())
            }
            else {
                (TestOutcome::Passed, Vec::new())
            }
        }
        Ok(Err(error)) => (TestOutcome::Failed, vec![error.into_test_failure()]),
        Err(payload) => (TestOutcome::Failed, vec![panic_failure(payload)]),
    };

    if outcome != TestOutcome::Skipped && expectation == Expectation::ExpectedFailure {
        if outcome == TestOutcome::Failed {
            outcome = TestOutcome::ExpectedFailure;
        }
        else {
            outcome = TestOutcome::Failed;
            failures.push(TestFailure::expectation(format!("expected failure for `{qualified_name}` but the body passed")));
        }
    }

    let skip_reason = context.skip_reason().map(str::to_owned);
    TestResult::new(descriptor, outcome, failures, started.elapsed(), skip_reason)
}

/// Executes `body` for the standard test harness.
///
/// The entry point panics with the rendered report when the result is a failure, which is how a
/// generated `#[test]` wrapper lets `cargo test` observe a structured failure. A passing, skipped,
/// or expected-failure result returns normally.
pub fn run<F, E>(descriptor: TestDescriptor, body: F)
where
    F: FnOnce(&mut TestContext) -> Result<(), E>,
    E: IntoTestFailure,
{
    let result = execute(descriptor, body);
    if !result.is_success() {
        std::panic::panic_any(result.render_text());
    }
}

/// Converts a caught panic payload into structured failure evidence.
fn panic_failure(payload: Box<dyn Any + Send>) -> TestFailure {
    match payload.downcast::<TestFailure>() {
        Ok(failure) => *failure,
        Err(payload) => {
            if let Some(message) = payload.downcast_ref::<&'static str>() {
                TestFailure::panic(*message)
            }
            else if let Some(message) = payload.downcast_ref::<String>() {
                TestFailure::panic(message.as_str())
            }
            else {
                TestFailure::panic("panic payload is not a string")
            }
        }
    }
}
