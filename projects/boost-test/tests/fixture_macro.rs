//! Verification that `#[boost_test::fixture]` and `#[boost_test::test(fixture = ...)]` compose.
//!
//! The generated wrappers are ordinary tests. The direct `execute_with_fixture` calls inspect the
//! structured result, because a passing wrapper cannot assert on evidence that is finalized after the
//! body returns.

#![cfg(feature = "macros")]

use std::sync::atomic::{AtomicUsize, Ordering};

use boost_test::{Fixture, FixtureDescriptor, Phase, SourceLocation, TestContext, TestDescriptor, TestFailure, TestOutcome, execute_with_fixture};

static SKIPPED_TEARDOWN: AtomicUsize = AtomicUsize::new(0);
static BROKEN_TEARDOWN: AtomicUsize = AtomicUsize::new(0);
static DIRECT_SKIPPED_TEARDOWN: AtomicUsize = AtomicUsize::new(0);

fn descriptor(name: &str) -> TestDescriptor {
    TestDescriptor::new(name, "tests::fixture_macro", SourceLocation::new("tests/fixture_macro.rs", 16, 1))
}

#[boost_test::fixture(teardown = release_resource)]
fn resource() -> Result<u32, String> {
    Ok(41)
}

fn release_resource(value: u32) {
    assert_eq!(value, 41);
}

#[boost_test::test(fixture = resource)]
fn fixture_reaches_a_context_body(context: &mut TestContext, value: &mut u32) {
    assert_eq!(*value, 41);
    assert_eq!(context.descriptor().name(), "fixture_reaches_a_context_body");
    assert_eq!(context.descriptor().case(), None);
}

#[boost_test::test(fixture = resource)]
fn fixture_reaches_a_fixture_only_body(value: &mut u32) -> Result<(), String> {
    boost_test::assert_eq!(*value, 41);
    Ok(())
}

#[boost_test::fixture]
fn named_resource() -> u8 {
    7
}

#[test]
fn fixture_descriptor_keeps_the_declared_name() {
    let descriptor: FixtureDescriptor = named_resource::descriptor();
    assert_eq!(descriptor.name(), "named_resource");
    assert!(descriptor.source().file().ends_with("fixture_macro.rs"));
    assert_eq!(descriptor.to_string(), "named_resource");
}

#[boost_test::fixture(teardown = release_skipped)]
fn skipping_resource(context: &mut TestContext) -> u32 {
    context.skip("the host has no backend");
    5
}

fn release_skipped(value: u32) {
    assert_eq!(value, 5);
    SKIPPED_TEARDOWN.fetch_add(1, Ordering::SeqCst);
}

#[boost_test::test(fixture = skipping_resource)]
fn skip_from_setup_suppresses_the_body(_value: &mut u32) {
    panic!("the body must not run after a setup skip");
}

#[boost_test::fixture(teardown = release_with_error)]
fn broken_resource() -> u8 {
    3
}

fn release_with_error(value: u8) -> Result<(), String> {
    assert_eq!(value, 3);
    BROKEN_TEARDOWN.fetch_add(1, Ordering::SeqCst);
    Err("release failed".to_owned())
}

#[test]
fn generated_fixture_flows_through_the_runner() {
    let result = execute_with_fixture::<resource, _, _>(descriptor("generated-fixture"), |_, value| {
        assert_eq!(*value, 41);
        Ok::<(), TestFailure>(())
    });
    assert_eq!(result.outcome(), TestOutcome::Passed);
    assert!(result.failures().is_empty());
}

#[test]
fn generated_result_teardown_is_recorded_after_the_body() {
    let result = execute_with_fixture::<broken_resource, _, _>(descriptor("broken-fixture"), |_, value| {
        assert_eq!(*value, 3);
        Ok::<(), TestFailure>(())
    });
    assert_eq!(result.outcome(), TestOutcome::Failed);
    assert_eq!(result.failures().len(), 1);
    assert_eq!(result.failures()[0].phase(), Phase::Teardown);
    assert_eq!(result.failures()[0].message(), "release failed");
    assert_eq!(BROKEN_TEARDOWN.load(Ordering::SeqCst), 1);
}

#[test]
fn setup_skip_still_releases_the_generated_fixture() {
    let result = execute_with_fixture::<direct_skipping_resource, _, _>(descriptor("skipping-fixture"), |_, _| -> Result<(), TestFailure> {
        panic!("the body must not run after a setup skip");
    });
    assert_eq!(result.outcome(), TestOutcome::Skipped);
    assert_eq!(result.skip_reason(), Some("the host has no backend"));
    assert_eq!(DIRECT_SKIPPED_TEARDOWN.load(Ordering::SeqCst), 1);
}

#[boost_test::fixture(teardown = release_direct_skipped)]
fn direct_skipping_resource(context: &mut TestContext) -> u32 {
    context.skip("the host has no backend");
    5
}

fn release_direct_skipped(value: u32) {
    assert_eq!(value, 5);
    DIRECT_SKIPPED_TEARDOWN.fetch_add(1, Ordering::SeqCst);
}
