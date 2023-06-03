//! Verification that `#[testkit::case]` expands one stable harness entry per declared case.

#![cfg(feature = "macros")]

#[testkit::case(
    "one plus one" => (1, 1),
    "two plus two" => (2, 2),
)]
fn addition(left: i32, right: i32) {
    testkit::assert_eq!(left + right, left * 2);
}

#[testkit::case("single positive" => 5, "single negative" => -5)]
fn single_parameter(value: i32) {
    testkit::assert!(value != 0);
}

#[testkit::case("text" => "alpha", "other text" => "beta")]
fn result_body(value: &str) -> Result<(), String> {
    if value.is_empty() { Err("empty value".to_owned()) } else { Ok(()) }
}

#[testkit::case("tuple parameter" => (7, 11))]
fn tuple_parameter(pair: (i32, i32)) {
    testkit::assert_eq!(pair.0 + pair.1, 18);
}
