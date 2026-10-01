//! SVG diagnostic preview rendering.
//!
//! This crate is reserved for structured diagnostic SVG output and is not implemented yet.

#![deny(missing_docs)]

/// Placeholder error for unimplemented SVG rendering.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderError {
    /// SVG rendering is not implemented yet.
    NotImplemented,
}
