//! Verification that `#[testkit::test]` generates a standard harness entry.
//!
//! The wrappers in this file are ordinary tests, so `cargo test` exercises them directly. A failing
//! wrapper is marked `#[ignore]` and invoked from another test through `catch_unwind`, which lets the
//! suite assert the structured report without keeping a permanently red entry.

#![cfg(feature = "macros")]

use std::panic::catch_unwind;

use testkit::TestContext;

#[testkit::test]
fn plain_body_passes() {
    testkit::assert_eq!(1 + 1, 2);
}

#[testkit::test(name = "renamed entry")]
fn renamed_body_passes() {}

#[testkit::test]
fn result_body_passes() -> Result<(), String> {
    Ok(())
}

#[testkit::test]
fn context_body_skips(context: &mut TestContext) {
    context.skip("no fixture on this host");
}

#[testkit::test]
fn result_body_with_context_passes(context: &mut TestContext) -> Result<(), String> {
    testkit::assert!(context.descriptor().name() == "result_body_with_context_passes");
    Ok(())
}

#[testkit::test(expected = "failure")]
fn declared_failure_with_expected_attribute() -> Result<(), String> {
    Err("declared".to_owned())
}

#[testkit::test(expected_failure)]
fn declared_failure_with_shorthand() -> Result<(), String> {
    Err("declared".to_owned())
}

#[testkit::test]
#[ignore = "invoked by another test to inspect the panic report"]
fn ignored_failing_body() {
    testkit::assert!(false, "wrapper failure");
}

#[test]
fn failing_wrapper_panics_with_the_structured_report() {
    let payload = catch_unwind(ignored_failing_body).expect_err("the wrapper must panic on a failed result");
    let report = payload.downcast_ref::<String>().expect("the wrapper panics with the text report");
    assert!(report.contains("ignored_failing_body ... FAILED"));
    assert!(report.contains("wrapper failure"));
    assert!(report.contains("[body condition]"));
}
