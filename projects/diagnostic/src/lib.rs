#![cfg_attr(not(feature = "std"), no_std)]
#![doc = include_str!("readme.md")]
#![deny(missing_docs)]

extern crate alloc;

pub mod collect;
pub mod model;
pub mod render;
pub mod source;

#[cfg(feature = "serde")]
pub mod wire;

#[cfg(feature = "terminal")]
pub mod terminal;

#[cfg(feature = "console")]
pub mod console_projection;

pub use collect::{DiagnosticSet, DiagnosticSink, SinkStatus};
pub use model::*;
pub use render::{diagnostic_message, message_fallback};
pub use source::SourceLookup;

#[cfg(feature = "console")]
pub use console_projection::{diagnostic_to_event, diagnostic_to_payload, emit_diagnostic, emit_diagnostic_set};

#[cfg(feature = "serde")]
pub use wire::{DiagnosticEnvelope, SCHEMA_VERSION};

#[cfg(feature = "terminal")]
pub use console::{Color, Console, Paint, Palette, Style};
#[cfg(feature = "terminal")]
pub use terminal::{
    enable_ansi_color, eprint_structured_set, structured_to_terminal, BuiltinDrawer, Config,
    Diagnostic as TerminalDiagnostic, DiagnosticBuilder as TerminalDiagnosticBuilder, DrawElements, Label,
    LabelAttach, ReportKind, ReportLevel, SourceRegistry, StructuredRenderError,
};
#[cfg(feature = "terminal")]
pub use terminal::{SourceCache, SourceID, SourceSpan};
