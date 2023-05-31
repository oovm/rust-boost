//! Verification of stable descriptor identity and case names.

use testkit::{Capability, CaseError, CaseId, Expectation, SourceLocation, TestDescriptor};

fn descriptor() -> TestDescriptor {
    TestDescriptor::new("adds", "crate::math", SourceLocation::new("tests/identity.rs", 12, 5))
}

#[test]
fn descriptor_identity_is_module_plus_name() {
    let descriptor = descriptor();
    assert_eq!(descriptor.name(), "adds");
    assert_eq!(descriptor.module(), "crate::math");
    assert_eq!(descriptor.qualified_name(), "crate::math::adds");
    assert_eq!(descriptor.to_string(), "crate::math::adds");
}

#[test]
fn descriptor_without_a_module_uses_the_bare_name() {
    let descriptor = TestDescriptor::new("alone", "", SourceLocation::new("src/lib.rs", 1, 1));
    assert_eq!(descriptor.qualified_name(), "alone");
}

#[test]
fn descriptor_starts_as_a_test_and_sorts_extra_capabilities() {
    let descriptor = descriptor().with_capability(Capability::Coverage).with_capability(Capability::Coverage);
    assert_eq!(descriptor.capabilities(), &[Capability::Test, Capability::Coverage]);
}

#[test]
fn expectation_is_explicit() {
    assert_eq!(descriptor().expectation(), Expectation::Pass);
    assert_eq!(descriptor().with_expectation(Expectation::ExpectedFailure).expectation(), Expectation::ExpectedFailure);
}

#[test]
fn source_location_is_rendered_as_file_line_column() {
    let location = SourceLocation::new("tests/identity.rs", 12, 5);
    assert_eq!(location.file(), "tests/identity.rs");
    assert_eq!(location.line(), 12);
    assert_eq!(location.column(), 5);
    assert_eq!(location.to_string(), "tests/identity.rs:12:5");
}

#[test]
fn case_names_must_not_be_empty() {
    assert_eq!(CaseId::new(""), Err(CaseError::Empty));
    let case = CaseId::new("input_is_empty").expect("a non-empty case name is valid");
    assert_eq!(case.as_str(), "input_is_empty");
    assert_eq!(case.to_string(), "input_is_empty");
}
