//! Procedural macro helpers for the `diagnostic` crate.

#![deny(missing_docs)]

mod expand;

use proc_macro::TokenStream;
use syn::parse_macro_input;

/// Build a structured [`diagnostic::Diagnostic`] record at the call site.
///
/// ```ignore
/// use diagnostic_macro::diagnose;
///
/// let record = diagnose! {
///     code: "oak.syntax.unexpected-token",
///     severity: error,
///     message: "unexpected token",
/// };
/// ```
#[proc_macro]
pub fn diagnose(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as expand::DiagnosticInput);
    expand::expand(parsed).into()
}
