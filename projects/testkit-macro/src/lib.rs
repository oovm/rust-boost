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
/// ```
#[proc_macro_attribute]
pub fn test(attribute: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as syn::ItemFn);
    let attribute: proc_macro2::TokenStream = attribute.into();
    expand::expand_test(attribute, item).unwrap_or_else(|error| error.to_compile_error()).into()
}
