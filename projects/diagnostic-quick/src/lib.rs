#![allow(clippy::needless_return)]
#![doc = include_str!("readme.md")]
#![deny(missing_docs)]

pub use diagnostic::{DiagnosticSet, Report, SourceID, SourceSpan};
pub use source_cache::SourceCache;

pub use self::convert::{qerror_to_structured, source_ref_for_id};
pub use self::errors::{
    display::print_errors, IOError, QError, QErrorKind, QResult, RuntimeError, SyntaxError,
};
pub use self::validation::Validation;

/// Conversion from quick errors to structured diagnostics.
pub mod convert;
/// Third-party error adapters.
pub mod error_3rd;
mod errors;
/// Legacy validation container.
mod validation;
