//! UTF-16 position mapping and structured diagnostic conversion for LSP clients.

mod convert;
mod error;
mod position;

pub use convert::{structured_to_lsp, SourceResolver};
pub use diagnostic;
pub use error::DiagnosticError;
pub use lsp_types;
pub use position::{
    byte_index_to_position, byte_range_to_lsp_range, byte_span_to_range, position_to_byte_index, range_to_byte_span,
};
pub use source_cache::{SourceCache, SourceID, SourceProvider, SourceSpan};
