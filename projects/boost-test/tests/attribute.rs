//! Verification that `#[boost_test::test]` generates a standard harness entry.
//!
//! The wrappers in this file are ordinary tests, so `cargo test` exercises them directly. A failing
//! wrapper is marked `#[ignore]` and invoked from another test through `catch_unwind`, which lets the
//! suite assert the structured report without keeping a permanently red entry.

#![cfg(feature = "macros")]

use std::panic::catch_unwind;

use boost_test::TestContext;

#[boost_test::test]
fn plain_body_passes() {
    boost_test::assert_eq!(1 + 1, 2);
}

#[boost_test::test(name = "renamed entry")]
fn renamed_body_passes() {}

#[boost_test::test]
fn result_body_passes() -> Result<(), String> {
    Ok(())
}

#[boost_test::test]
fn context_body_skips(context: &mut TestContext) {
    context.skip("no fixture on this host");
}

#[boost_test::test]
fn result_body_with_context_passes(context: &mut TestContext) -> Result<(), String> {
    boost_test::assert!(context.descriptor().name() == "result_body_with_context_passes");
    Ok(())
}

#[boost_test::test(case = "empty table")]
fn case_labelled_body_passes(context: &mut TestContext) {
    boost_test::assert_eq!(context.descriptor().case().map(boost_test::CaseId::as_str), Some("empty table"));
    boost_test::assert!(context.descriptor().qualified_name().ends_with("[empty table]"));
}

#[boost_test::test(expected = "failure")]
fn declared_failure_with_expected_attribute() -> Result<(), String> {
    Err("declared".to_owned())
}

#[boost_test::test(expected_failure)]
fn declared_failure_with_shorthand() -> Result<(), String> {
    Err("declared".to_owned())
}

#[boost_test::test]
#[ignore = "invoked by another test to inspect the panic report"]
fn ignored_failing_body() {
    boost_test::assert!(false, "wrapper failure");
}

#[test]
fn failing_wrapper_panics_with_the_structured_report() {
    let payload = catch_unwind(ignored_failing_body).expect_err("the wrapper must panic on a failed result");
    let report = payload.downcast_ref::<String>().expect("the wrapper panics with the text report");
    assert!(report.contains("ignored_failing_body ... FAILED"));
    assert!(report.contains("wrapper failure"));
    assert!(report.contains("[body condition]"));
}
