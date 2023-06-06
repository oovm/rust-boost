//! Verification of structured assertion failures.
//!
//! An assertion is checked through `execute`, which is the same conversion the standard harness entry
//! uses. Each case asserts the failure kind, the source location, and the ordered operands, so a
//! regression that flattens the payload into text is visible even when the body still aborts.

use boost_test::{AssertionKind, FailureKind, SourceLocation, TestDescriptor, TestFailure, TestOutcome, execute};

/// Executes an assertion body and returns the structured result.
fn capture(body: impl FnOnce()) -> boost_test::TestResult {
    let descriptor = TestDescriptor::new("assertion", "tests::assertions", SourceLocation::new("tests/assertions.rs", 1, 1));
    execute(descriptor, |_| {
        body();
        Ok::<(), TestFailure>(())
    })
}

/// Returns the single failure of a failed result.
fn single_failure(result: &boost_test::TestResult) -> &TestFailure {
    assert_eq!(result.outcome(), TestOutcome::Failed);
    assert_eq!(result.failures().len(), 1);
    &result.failures()[0]
}

#[test]
fn passing_assertions_do_not_fail_the_body() {
    let result = capture(|| {
        boost_test::assert!(1 + 1 == 2);
        boost_test::assert_eq!(2, 2);
        boost_test::assert_ne!(1, 2);
    });
    assert_eq!(result.outcome(), TestOutcome::Passed);
    assert!(result.failures().is_empty());
}

#[test]
fn assert_equal_keeps_both_operands_and_the_location() {
    let result = capture(|| boost_test::assert_eq!(1, 2));
    let failure = single_failure(&result);
    assert_eq!(failure.kind(), FailureKind::Assertion(AssertionKind::Equal));
    assert_eq!(failure.message(), "assertion failed: left == right");
    assert_eq!(failure.operands().len(), 2);
    assert_eq!(failure.operands()[0].label(), "left");
    assert_eq!(failure.operands()[0].value().as_str(), "1");
    assert_eq!(failure.operands()[1].label(), "right");
    assert_eq!(failure.operands()[1].value().as_str(), "2");
    let location = failure.location().expect("an assertion captures its source location");
    assert!(location.file().ends_with("assertions.rs"));
    assert!(location.line() > 0);
    assert!(location.column() > 0);
}

#[test]
fn assert_equal_uses_the_custom_message() {
    let result = capture(|| boost_test::assert_eq!(1, 2, "left and right diverged"));
    let failure = single_failure(&result);
    assert_eq!(failure.message(), "left and right diverged");
}

#[test]
fn assert_not_equal_reports_the_not_equal_kind() {
    let result = capture(|| boost_test::assert_ne!(3, 3));
    let failure = single_failure(&result);
    assert_eq!(failure.kind(), FailureKind::Assertion(AssertionKind::NotEqual));
    assert_eq!(failure.message(), "assertion failed: left != right");
}

#[test]
fn assert_condition_reports_the_condition_kind() {
    let result = capture(|| boost_test::assert!(false, "condition must hold: {}", 7));
    let failure = single_failure(&result);
    assert_eq!(failure.kind(), FailureKind::Assertion(AssertionKind::Condition));
    assert_eq!(failure.message(), "condition must hold: 7");
    assert!(failure.operands().is_empty());
}

#[test]
fn explicit_failure_reports_the_fail_kind() {
    let result = capture(|| boost_test::fail!("stop after {}", 3));
    let failure = single_failure(&result);
    assert_eq!(failure.kind(), FailureKind::Assertion(AssertionKind::Fail));
    assert_eq!(failure.message(), "stop after 3");
    assert!(failure.operands().is_empty());
}

#[test]
fn text_projection_renders_the_structured_operands() {
    let result = capture(|| boost_test::assert_eq!(1, 2));
    let text = result.render_text();
    assert!(text.contains("FAILED"));
    assert!(text.contains("[body equal]"));
    assert!(text.contains("assertion failed: left == right"));
    assert!(text.contains("left: 1"));
    assert!(text.contains("right: 2"));
}
