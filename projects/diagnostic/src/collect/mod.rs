//! Diagnostic collection and sink interfaces.

mod set;
mod sink;

pub use set::DiagnosticSet;
pub use sink::{DiagnosticSink, SinkStatus};
