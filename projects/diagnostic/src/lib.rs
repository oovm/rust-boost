#![cfg_attr(not(feature = "std"), no_std)]
#![doc = include_str!("readme.md")]
#![deny(missing_docs)]

extern crate alloc;

pub mod model;
pub mod report;
pub mod set;

#[cfg(feature = "serde")]
pub mod json;

#[cfg(feature = "terminal")]
pub mod terminal;

pub use model::*;
pub use report::Report;
pub use set::{DiagnosticEmitter, DiagnosticSet, EmitStatus};

#[cfg(feature = "terminal")]
pub use terminal::{
    enable_ansi_color, eprint_structured_set, structured_to_terminal, BuiltinDrawer, Color, Config, Console,
    Diagnostic as TerminalDiagnostic, DiagnosticBuilder as TerminalDiagnosticBuilder, DrawElements, Label,
    LabelAttach, Paint, Palette, ReportKind, ReportLevel, SourceRegistry, StructuredRenderError, Style,
};
#[cfg(feature = "terminal")]
pub use terminal::{SourceCache, SourceID, SourceSpan};
