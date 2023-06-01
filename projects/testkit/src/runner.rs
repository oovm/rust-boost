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
    fixture::Fixture,
    result::{IntoTestFailure, Phase, TestFailure, TestOutcome, TestResult},
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

    outcome = apply_expectation(outcome, &mut failures, expectation, &qualified_name);

    let skip_reason = context.skip_reason().map(str::to_owned);
    TestResult::new(descriptor, outcome, failures, started.elapsed(), skip_reason)
}

/// Executes `body` with a fixture and returns the structured result.
///
/// The runner owns the phase order. Setup failure is recorded as a [`Phase::Setup`] failure and the
/// body is not entered. A skipped context suppresses the body but still releases the fixture.
/// Teardown runs after a passing, failing, or panicking body, and its failure is appended after the
/// original body failure instead of replacing it.
///
/// A failure observed outside the body keeps the result failed even when the descriptor declares an
/// expected failure: only body failures can satisfy that expectation.
pub fn execute_with_fixture<F, B, E>(descriptor: TestDescriptor, body: B) -> TestResult
where
    F: Fixture,
    B: FnOnce(&mut TestContext, &mut F::Value) -> Result<(), E>,
    E: IntoTestFailure,
{
    let expectation = descriptor.expectation();
    let qualified_name = descriptor.qualified_name();
    let started = Instant::now();
    let mut context = TestContext::new(descriptor.clone());
    let mut failures = Vec::new();

    let setup = catch_unwind(AssertUnwindSafe(|| F::setup(&mut context)));
    let value = match setup {
        Ok(Ok(value)) => Some(value),
        Ok(Err(failure)) => {
            failures.push(failure.with_phase(Phase::Setup));
            None
        }
        Err(payload) => {
            failures.push(panic_failure(payload).with_phase(Phase::Setup));
            None
        }
    };

    if let Some(mut value) = value {
        if !context.is_skipped() {
            match catch_unwind(AssertUnwindSafe(|| body(&mut context, &mut value))) {
                Ok(Ok(())) => {}
                Ok(Err(error)) => failures.push(error.into_test_failure().with_phase(Phase::Body)),
                Err(payload) => failures.push(panic_failure(payload).with_phase(Phase::Body)),
            }
        }
        match catch_unwind(AssertUnwindSafe(|| F::teardown(value))) {
            Ok(Ok(())) => {}
            Ok(Err(failure)) => failures.push(failure.with_phase(Phase::Teardown)),
            Err(payload) => failures.push(panic_failure(payload).with_phase(Phase::Teardown)),
        }
    }

    let mut outcome =
        if failures.is_empty() { if context.is_skipped() { TestOutcome::Skipped } else { TestOutcome::Passed } } else { TestOutcome::Failed };
    outcome = apply_expectation(outcome, &mut failures, expectation, &qualified_name);

    let skip_reason = context.skip_reason().map(str::to_owned);
    TestResult::new(descriptor, outcome, failures, started.elapsed(), skip_reason)
}

/// Executes `body` with a fixture for the standard test harness.
///
/// The function panics with the rendered report when the result is a failure, exactly like
/// [`run`]. A passing, skipped, or expected-failure result returns normally.
pub fn run_with_fixture<F, B, E>(descriptor: TestDescriptor, body: B)
where
    F: Fixture,
    B: FnOnce(&mut TestContext, &mut F::Value) -> Result<(), E>,
    E: IntoTestFailure,
{
    let result = execute_with_fixture::<F, B, E>(descriptor, body);
    if !result.is_success() {
        std::panic::panic_any(result.render_text());
    }
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

/// Applies the descriptor expectation to the observed outcome.
///
/// A skip keeps precedence. An expected failure is satisfied only by failures recorded in the body,
/// so a setup or teardown failure cannot be hidden behind the declaration.
fn apply_expectation(outcome: TestOutcome, failures: &mut Vec<TestFailure>, expectation: Expectation, qualified_name: &str) -> TestOutcome {
    if outcome == TestOutcome::Skipped || expectation != Expectation::ExpectedFailure {
        return outcome;
    }
    match outcome {
        TestOutcome::Failed if failures.iter().all(|failure| failure.phase() == Phase::Body) => TestOutcome::ExpectedFailure,
        TestOutcome::Failed => TestOutcome::Failed,
        _ => {
            failures.push(TestFailure::expectation(format!("expected failure for `{qualified_name}` but the body passed")));
            TestOutcome::Failed
        }
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
