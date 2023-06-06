//! Verification of the fixture lifecycle: setup, body, teardown, failure phases, and skip.

use std::{
    panic::catch_unwind,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

use boost_test::{
    Expectation, Fixture, FixtureDescriptor, Phase, SourceLocation, TestContext, TestDescriptor, TestFailure, TestOutcome,
    execute_with_fixture, run_with_fixture,
};

fn descriptor(name: &str) -> TestDescriptor {
    TestDescriptor::new(name, "tests::fixture", SourceLocation::new("tests/fixture.rs", 12, 1))
}

/// A fixture whose value reaches the body and whose teardown observes the same value.
struct Passing;

static PASSING_TEARDOWN: AtomicUsize = AtomicUsize::new(0);

impl Fixture for Passing {
    type Value = u32;

    fn descriptor() -> FixtureDescriptor {
        FixtureDescriptor::new("passing", SourceLocation::new("tests/fixture.rs", 22, 1))
    }

    fn setup(_: &mut TestContext) -> Result<Self::Value, TestFailure> {
        Ok(7)
    }

    fn teardown(value: Self::Value) -> Result<(), TestFailure> {
        assert_eq!(value, 7);
        PASSING_TEARDOWN.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[test]
fn fixture_runs_setup_body_and_teardown() {
    let result = execute_with_fixture::<Passing, _, _>(descriptor("fixture-order"), |_, value| {
        assert_eq!(*value, 7);
        Ok::<(), TestFailure>(())
    });
    assert_eq!(result.outcome(), TestOutcome::Passed);
    assert!(result.failures().is_empty());
    assert_eq!(PASSING_TEARDOWN.load(Ordering::SeqCst), 1);
}

/// A fixture whose setup returns a structured failure instead of a value.
struct FailingSetup;

static FAILING_SETUP_BODY_RUN: AtomicBool = AtomicBool::new(false);

impl Fixture for FailingSetup {
    type Value = u32;

    fn descriptor() -> FixtureDescriptor {
        FixtureDescriptor::new("failing-setup", SourceLocation::new("tests/fixture.rs", 48, 1))
    }

    fn setup(_: &mut TestContext) -> Result<Self::Value, TestFailure> {
        Err(TestFailure::returned("no backend"))
    }

    fn teardown(_: Self::Value) -> Result<(), TestFailure> {
        Ok(())
    }
}

#[test]
fn setup_failure_is_recorded_in_the_setup_phase_and_skips_the_body() {
    let result = execute_with_fixture::<FailingSetup, _, _>(descriptor("setup-failure"), |_, _| {
        FAILING_SETUP_BODY_RUN.store(true, Ordering::SeqCst);
        Ok::<(), TestFailure>(())
    });
    assert_eq!(result.outcome(), TestOutcome::Failed);
    assert_eq!(result.failures().len(), 1);
    assert_eq!(result.failures()[0].phase(), Phase::Setup);
    assert_eq!(result.failures()[0].message(), "no backend");
    assert!(!FAILING_SETUP_BODY_RUN.load(Ordering::SeqCst));
}

/// A fixture that panics during setup.
struct PanickingSetup;

impl Fixture for PanickingSetup {
    type Value = u32;

    fn descriptor() -> FixtureDescriptor {
        FixtureDescriptor::new("panicking-setup", SourceLocation::new("tests/fixture.rs", 78, 1))
    }

    fn setup(_: &mut TestContext) -> Result<Self::Value, TestFailure> {
        panic!("setup exploded")
    }

    fn teardown(_: Self::Value) -> Result<(), TestFailure> {
        Ok(())
    }
}

#[test]
fn setup_panic_is_recorded_in_the_setup_phase() {
    let result = execute_with_fixture::<PanickingSetup, _, _>(descriptor("setup-panic"), |_, _| Ok::<(), TestFailure>(()));
    assert_eq!(result.outcome(), TestOutcome::Failed);
    assert_eq!(result.failures()[0].phase(), Phase::Setup);
    assert_eq!(result.failures()[0].kind(), boost_test::FailureKind::Panic);
    assert_eq!(result.failures()[0].message(), "setup exploded");
}

/// A fixture whose teardown succeeds while the body fails.
struct BodyFailure;

static BODY_FAILURE_TEARDOWN: AtomicUsize = AtomicUsize::new(0);

impl Fixture for BodyFailure {
    type Value = u8;

    fn descriptor() -> FixtureDescriptor {
        FixtureDescriptor::new("body-failure", SourceLocation::new("tests/fixture.rs", 108, 1))
    }

    fn setup(_: &mut TestContext) -> Result<Self::Value, TestFailure> {
        Ok(1)
    }

    fn teardown(_: Self::Value) -> Result<(), TestFailure> {
        BODY_FAILURE_TEARDOWN.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[test]
fn teardown_runs_after_body_failure_and_keeps_the_body_failure_first() {
    let result = execute_with_fixture::<BodyFailure, _, _>(descriptor("body-failure"), |_, _| Err::<(), _>("body failed"));
    assert_eq!(result.outcome(), TestOutcome::Failed);
    assert_eq!(result.failures().len(), 1);
    assert_eq!(result.failures()[0].phase(), Phase::Body);
    assert_eq!(result.failures()[0].message(), "body failed");
    assert_eq!(BODY_FAILURE_TEARDOWN.load(Ordering::SeqCst), 1);
}

/// A fixture whose teardown returns a failure.
struct TeardownFailure;

impl Fixture for TeardownFailure {
    type Value = u8;

    fn descriptor() -> FixtureDescriptor {
        FixtureDescriptor::new("teardown-failure", SourceLocation::new("tests/fixture.rs", 139, 1))
    }

    fn setup(_: &mut TestContext) -> Result<Self::Value, TestFailure> {
        Ok(1)
    }

    fn teardown(_: Self::Value) -> Result<(), TestFailure> {
        Err(TestFailure::returned("cleanup failed"))
    }
}

#[test]
fn teardown_failure_is_appended_after_a_passing_body() {
    let result = execute_with_fixture::<TeardownFailure, _, _>(descriptor("teardown-failure"), |_, _| Ok::<(), TestFailure>(()));
    assert_eq!(result.outcome(), TestOutcome::Failed);
    assert_eq!(result.failures().len(), 1);
    assert_eq!(result.failures()[0].phase(), Phase::Teardown);
    assert_eq!(result.failures()[0].kind(), boost_test::FailureKind::Returned);
    assert_eq!(result.failures()[0].message(), "cleanup failed");
}

#[test]
fn teardown_failure_cannot_be_hidden_by_an_expected_failure() {
    let descriptor = descriptor("expected-with-teardown").with_expectation(Expectation::ExpectedFailure);
    let result = execute_with_fixture::<TeardownFailure, _, _>(descriptor, |_, _| Err::<(), _>("body failed"));
    assert_eq!(result.outcome(), TestOutcome::Failed);
    assert_eq!(result.failures().len(), 2);
    assert_eq!(result.failures()[0].phase(), Phase::Body);
    assert_eq!(result.failures()[1].phase(), Phase::Teardown);
}

/// A fixture whose body panics.
struct PanickingBody;

static PANICKING_BODY_TEARDOWN: AtomicUsize = AtomicUsize::new(0);

impl Fixture for PanickingBody {
    type Value = u8;

    fn descriptor() -> FixtureDescriptor {
        FixtureDescriptor::new("panicking-body", SourceLocation::new("tests/fixture.rs", 181, 1))
    }

    fn setup(_: &mut TestContext) -> Result<Self::Value, TestFailure> {
        Ok(1)
    }

    fn teardown(_: Self::Value) -> Result<(), TestFailure> {
        PANICKING_BODY_TEARDOWN.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[test]
fn body_panic_still_runs_teardown() {
    let result = execute_with_fixture::<PanickingBody, _, _>(descriptor("panicking-body"), |_, _| -> Result<(), TestFailure> {
        panic!("body exploded")
    });
    assert_eq!(result.outcome(), TestOutcome::Failed);
    assert_eq!(result.failures()[0].phase(), Phase::Body);
    assert_eq!(result.failures()[0].message(), "body exploded");
    assert_eq!(PANICKING_BODY_TEARDOWN.load(Ordering::SeqCst), 1);
}

/// A fixture that requests a skip during setup.
struct SkippingSetup;

static SKIPPING_SETUP_BODY_RUN: AtomicBool = AtomicBool::new(false);
static SKIPPING_SETUP_TEARDOWN: AtomicUsize = AtomicUsize::new(0);

impl Fixture for SkippingSetup {
    type Value = u8;

    fn descriptor() -> FixtureDescriptor {
        FixtureDescriptor::new("skipping-setup", SourceLocation::new("tests/fixture.rs", 216, 1))
    }

    fn setup(context: &mut TestContext) -> Result<Self::Value, TestFailure> {
        context.skip("the host has no backend");
        Ok(1)
    }

    fn teardown(_: Self::Value) -> Result<(), TestFailure> {
        SKIPPING_SETUP_TEARDOWN.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[test]
fn skip_during_setup_suppresses_the_body_but_releases_the_fixture() {
    let result = execute_with_fixture::<SkippingSetup, _, _>(descriptor("skipping-setup"), |_, _| {
        SKIPPING_SETUP_BODY_RUN.store(true, Ordering::SeqCst);
        Ok::<(), TestFailure>(())
    });
    assert_eq!(result.outcome(), TestOutcome::Skipped);
    assert!(result.is_success());
    assert_eq!(result.skip_reason(), Some("the host has no backend"));
    assert!(!SKIPPING_SETUP_BODY_RUN.load(Ordering::SeqCst));
    assert_eq!(SKIPPING_SETUP_TEARDOWN.load(Ordering::SeqCst), 1);
}

/// A fixture whose body requests a skip.
struct SkippingBody;

static SKIPPING_BODY_TEARDOWN: AtomicUsize = AtomicUsize::new(0);

impl Fixture for SkippingBody {
    type Value = u8;

    fn descriptor() -> FixtureDescriptor {
        FixtureDescriptor::new("skipping-body", SourceLocation::new("tests/fixture.rs", 252, 1))
    }

    fn setup(_: &mut TestContext) -> Result<Self::Value, TestFailure> {
        Ok(1)
    }

    fn teardown(_: Self::Value) -> Result<(), TestFailure> {
        SKIPPING_BODY_TEARDOWN.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[test]
fn skip_in_the_body_still_releases_the_fixture() {
    let result = execute_with_fixture::<SkippingBody, _, _>(descriptor("skipping-body"), |context, _| {
        context.skip("nothing to verify");
        Ok::<(), TestFailure>(())
    });
    assert_eq!(result.outcome(), TestOutcome::Skipped);
    assert_eq!(result.skip_reason(), Some("nothing to verify"));
    assert_eq!(SKIPPING_BODY_TEARDOWN.load(Ordering::SeqCst), 1);
}

#[test]
fn run_with_fixture_panics_with_the_rendered_report_on_failure() {
    let payload = catch_unwind(|| run_with_fixture::<FailingSetup, _, _>(descriptor("run-fixture"), |_, _| Ok::<(), TestFailure>(())))
        .expect_err("a failed result must panic the harness entry");
    let report = payload.downcast_ref::<String>().expect("the harness entry panics with the text report");
    assert!(report.contains("test tests::fixture::run-fixture ... FAILED"));
    assert!(report.contains("[setup returned]"));
    assert!(report.contains("no backend"));
}

#[test]
fn run_with_fixture_returns_normally_for_success() {
    run_with_fixture::<Runnable, _, _>(descriptor("run-fixture-ok"), |_, value| {
        assert_eq!(*value, 7);
        Ok::<(), TestFailure>(())
    });
    assert_eq!(RUNNABLE_TEARDOWN.load(Ordering::SeqCst), 1);
}

#[test]
fn fixture_descriptor_keeps_its_declared_identity() {
    let descriptor = Passing::descriptor();
    assert_eq!(descriptor.name(), "passing");
    assert_eq!(descriptor.source().file(), "tests/fixture.rs");
    assert_eq!(descriptor.source().line(), 22);
    assert_eq!(descriptor.to_string(), "passing");
}

/// A fixture used only by the harness-entry success test.
struct Runnable;

static RUNNABLE_TEARDOWN: AtomicUsize = AtomicUsize::new(0);

impl Fixture for Runnable {
    type Value = u32;

    fn descriptor() -> FixtureDescriptor {
        FixtureDescriptor::new("runnable", SourceLocation::new("tests/fixture.rs", 304, 1))
    }

    fn setup(_: &mut TestContext) -> Result<Self::Value, TestFailure> {
        Ok(7)
    }

    fn teardown(value: Self::Value) -> Result<(), TestFailure> {
        assert_eq!(value, 7);
        RUNNABLE_TEARDOWN.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}
