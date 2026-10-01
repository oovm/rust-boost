#![allow(clippy::needless_return)]
#![doc = include_str!("readme.md")]
#![deny(missing_docs)]

pub use diagnostic::{
    DiagnosticLevel, SourceID, Span, TextStorage,
    Validation::{Failure, Success},
};

pub use self::errors::{display::print_errors, IOError, QError, QErrorKind, QResult, RuntimeError, SyntaxError, Validation};

pub mod error_3rd;
mod errors;
// #[cfg(feature = "lsp-types")]
// mod for_lsp;
