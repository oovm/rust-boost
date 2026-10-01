//! Procedural macro helpers for the `diagnostic` crate.

#![deny(missing_docs)]

use proc_macro::TokenStream;

/// Compile-time diagnostic hook used by integration tests.
///
/// The macro currently expands to an empty expression. Future versions may emit
/// structured diagnostics during macro expansion.
#[proc_macro]
pub fn real_macro(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}
