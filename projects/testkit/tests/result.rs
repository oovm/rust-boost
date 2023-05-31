//! Verification of the result contract: outcome, failures, skip, expectation, and text projection.
//!
//! The tests drive `execute` directly, so the phase order, the panic conversion, and the expectation
//! rewrite are observed as structured facts instead of inferred from the standard harness exit code.

use std::panic::catch_unwind;

use testkit::{Expectation, FailureKind, SourceLocation, TestDescriptor, TestFailure, TestOutcome, execute, run};

/// Builds a descriptor whose module is fixed, so the qualified name is deterministic.
fn descriptor(name: &str) -> TestDescriptor {
    TestDescriptor::new(name, "tests::result", SourceLocation::new("tests/result.rs", 7, 3))
}

#[test]
fn passing_body_reports_passed() {
    let result = execute(descriptor("passing"), |_| Ok::<(), TestFailure>(()));
    assert_eq!(result.outcome(), TestOutcome::Passed);
    assert!(result.is_success());
    assert!(result.failures().is_empty());
    assert_eq!(result.skip_reason(), None);
    assert!(result.render_text().contains("test tests::result::passing ... ok"));
}

#[test]
fn returned_error_reports_body_failure() {
    let result = execute(descriptor("returned"), |_| Err::<(), _>("the body refused"));
    assert_eq!(result.outcome(), TestOutcome::Failed);
    assert!(!result.is_success());
    assert_eq!(result.failures().len(), 1);
    let failure = &result.failures()[0];
    assert_eq!(failure.kind(), FailureKind::Returned);
    assert_eq!(failure.message(), "the body refused");
    assert_eq!(failure.location(), None);
}

#[test]
fn panic_reports_body_failure_and_keeps_the_message() {
    let result = execute(descriptor("panic"), |_| -> Result<(), TestFailure> { panic!("the body exploded") });
    assert_eq!(result.outcome(), TestOutcome::Failed);
    assert_eq!(result.failures().len(), 1);
    let failure = &result.failures()[0];
    assert_eq!(failure.kind(), FailureKind::Panic);
    assert_eq!(failure.message(), "the body exploded");
}

#[test]
fn skip_takes_precedence_over_expectation() {
    let descriptor = descriptor("skipped").with_expectation(Expectation::ExpectedFailure);
    let result = execute(descriptor, |context| {
        context.skip("the host has no backend");
        Ok::<(), TestFailure>(())
    });
    assert_eq!(result.outcome(), TestOutcome::Skipped);
    assert!(result.is_success());
    assert!(result.failures().is_empty());
    assert_eq!(result.skip_reason(), Some("the host has no backend"));
    assert!(result.render_text().contains("skipped: the host has no backend"));
}

#[test]
fn expected_failure_keeps_the_failure_evidence() {
    let result = execute(descriptor("expected").with_expectation(Expectation::ExpectedFailure), |_| Err::<(), _>("declared failure"));
    assert_eq!(result.outcome(), TestOutcome::ExpectedFailure);
    assert!(result.is_success());
    assert_eq!(result.failures().len(), 1);
    assert_eq!(result.failures()[0].message(), "declared failure");
    assert!(result.render_text().contains("expected failure"));
}

#[test]
fn expected_failure_that_passes_is_a_failure() {
    let result = execute(descriptor("missing").with_expectation(Expectation::ExpectedFailure), |_| Ok::<(), TestFailure>(()));
    assert_eq!(result.outcome(), TestOutcome::Failed);
    assert_eq!(result.failures().len(), 1);
    let failure = &result.failures()[0];
    assert_eq!(failure.kind(), FailureKind::Expectation);
    assert!(failure.message().contains("expected failure"));
    assert!(failure.message().contains("tests::result::missing"));
}

#[test]
fn run_panics_with_the_rendered_report_on_failure() {
    let payload = catch_unwind(|| run(descriptor("run-failure"), |_| Err::<(), _>("body failed")))
        .expect_err("a failed result must panic the harness entry");
    let report = payload.downcast_ref::<String>().expect("the harness entry panics with the text report");
    assert!(report.contains("test tests::result::run-failure ... FAILED"));
    assert!(report.contains("body failed"));
}

#[test]
fn run_returns_normally_for_success_and_skip() {
    run(descriptor("run-ok"), |_| Ok::<(), TestFailure>(()));
    run(descriptor("run-skip"), |context| {
        context.skip("nothing to do");
        Ok::<(), TestFailure>(())
    });
}

#[test]
fn failure_phase_defaults_to_the_body() {
    let failure = TestFailure::returned("phase");
    assert_eq!(failure.phase(), testkit::Phase::Body);
}
