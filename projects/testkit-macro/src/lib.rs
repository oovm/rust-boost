//! Procedural macros for `testkit`.

#![deny(missing_docs)]

mod expand;

use proc_macro::TokenStream;
use syn::parse_macro_input;

/// Wraps a free function as a standard `#[test]` entry with a structured `testkit` descriptor.
///
/// The generated wrapper calls [`testkit::run`](https://docs.rs/testkit) and keeps the original body
/// as an ordinary function, so `?`, `return`, and panic behaviour are unchanged. A body may take no
/// arguments or one `&mut testkit::TestContext` and may return `()` or `Result<(), E>`.
///
/// With `fixture = path`, the wrapper calls
/// [`testkit::run_with_fixture`](https://docs.rs/testkit) instead. The body then takes the fixture as
/// a trailing `&mut Value`, optionally after `&mut TestContext`.
///
/// ```ignore
/// #[testkit::test]
/// fn adds_two_numbers() {
///     testkit::assert_eq!(1 + 1, 2);
/// }
///
/// #[testkit::test(name = "renamed entry", expected = "failure")]
/// fn declared_failure() -> Result<(), String> {
///     Err("expected".to_owned())
/// }
///
/// #[testkit::test(fixture = database, case = "empty table")]
/// fn reads_an_empty_table(context: &mut testkit::TestContext, database: &mut Database) {
///     testkit::assert!(database.is_empty());
/// }
/// ```
#[proc_macro_attribute]
pub fn test(attribute: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as syn::ItemFn);
    let attribute: proc_macro2::TokenStream = attribute.into();
    expand::expand_test(attribute, item).unwrap_or_else(|error| error.to_compile_error()).into()
}

/// Declares a typed fixture from a setup function and an optional teardown path.
///
/// The setup function may take no arguments or one `&mut testkit::TestContext`, and returns either
/// the fixture value or `Result<Value, E>`. The macro keeps the setup function callable and adds a
/// marker type with the same name in the type namespace, so `fixture = name` can name it.
///
/// A teardown named with `teardown = path` is called with the fixture value and may return `()` or
/// `Result<(), E>`. It runs after the body, including after a body panic.
///
/// ```ignore
/// #[testkit::fixture(teardown = release_database)]
/// fn database() -> Result<Database, DatabaseError> {
///     Database::open()
/// }
///
/// fn release_database(database: Database) {
///     database.close();
/// }
/// ```
#[proc_macro_attribute]
pub fn fixture(attribute: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as syn::ItemFn);
    let attribute: proc_macro2::TokenStream = attribute.into();
    expand::expand_fixture(attribute, item).unwrap_or_else(|error| error.to_compile_error()).into()
}

/// Expands one parameterized body into a standard `#[test]` entry per declared case.
///
/// Each entry is `"stable case name" => arguments`, separated by commas. A body with several
/// parameters receives a tuple with exactly that many elements; a body with one parameter receives
/// the expression itself. Every case keeps its name in the descriptor identity, so a report can
/// distinguish two runs of the same body.
///
/// ```ignore
/// #[testkit::case(
///     "one plus one" => (1, 1),
///     "two plus two" => (2, 2),
/// )]
/// fn addition(left: i32, right: i32) {
///     testkit::assert_eq!(left + right, left * 2);
/// }
/// ```
#[proc_macro_attribute]
pub fn case(attribute: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as syn::ItemFn);
    let attribute: proc_macro2::TokenStream = attribute.into();
    expand::expand_case(attribute, item).unwrap_or_else(|error| error.to_compile_error()).into()
}
