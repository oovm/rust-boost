//! Procedural macros for `testkit`.

#![deny(missing_docs)]

mod expand;

use expand::RuntimePath;
use proc_macro::TokenStream;
use syn::parse_macro_input;

fn expand_with_runtime(
    attribute: TokenStream,
    item: TokenStream,
    expand: fn(proc_macro2::TokenStream, syn::ItemFn, RuntimePath) -> syn::Result<proc_macro2::TokenStream>,
    default_runtime: RuntimePath,
) -> TokenStream {
    let item = parse_macro_input!(item as syn::ItemFn);
    let attribute: proc_macro2::TokenStream = attribute.into();
    expand(attribute, item, default_runtime).unwrap_or_else(|error| error.to_compile_error()).into()
}

/// Wraps a free function as a standard `#[test]` entry with a structured `testkit` descriptor.
///
/// Generated code references `::boost_test` by default. Use the facade re-export
/// `boost::test::test` or `runtime = ::boost::test` when depending only on `boost`.
#[proc_macro_attribute]
pub fn test(attribute: TokenStream, item: TokenStream) -> TokenStream {
    expand_with_runtime(attribute, item, expand::expand_test, RuntimePath::direct())
}

/// Facade entry that expands against `::boost::test`.
#[proc_macro_attribute]
pub fn facade_test(attribute: TokenStream, item: TokenStream) -> TokenStream {
    expand_with_runtime(attribute, item, expand::expand_test, RuntimePath::facade())
}

/// Declares a typed fixture from a setup function and an optional teardown path.
#[proc_macro_attribute]
pub fn fixture(attribute: TokenStream, item: TokenStream) -> TokenStream {
    expand_with_runtime(attribute, item, expand::expand_fixture, RuntimePath::direct())
}

/// Facade entry that expands against `::boost::test`.
#[proc_macro_attribute]
pub fn facade_fixture(attribute: TokenStream, item: TokenStream) -> TokenStream {
    expand_with_runtime(attribute, item, expand::expand_fixture, RuntimePath::facade())
}

/// Expands one parameterized body into a standard `#[test]` entry per declared case.
#[proc_macro_attribute]
pub fn case(attribute: TokenStream, item: TokenStream) -> TokenStream {
    expand_with_runtime(attribute, item, expand::expand_case, RuntimePath::direct())
}

/// Facade entry that expands against `::boost::test`.
#[proc_macro_attribute]
pub fn facade_case(attribute: TokenStream, item: TokenStream) -> TokenStream {
    expand_with_runtime(attribute, item, expand::expand_case, RuntimePath::facade())
}
