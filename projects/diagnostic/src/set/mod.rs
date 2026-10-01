//! Ordered diagnostic collection and emission.

mod emitter;
mod set;

pub use emitter::{DiagnosticEmitter, EmitStatus};
pub use set::DiagnosticSet;
