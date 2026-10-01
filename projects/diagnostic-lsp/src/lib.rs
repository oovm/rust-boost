//! Utilities for translating from codespan types into Language Server Protocol (LSP) types
#![deny(missing_docs)]

// WARNING: Be extremely careful when adding new imports here, as it could break
// the compatible version range that we claim in our `Cargo.toml`. This could
// potentially break down-stream builds on a `cargo update`. This is an
// absolute no-no, breaking much of what we enjoy about Cargo!
use lsp_types::{Position, Range};

use diagnostic::{SourceCache, SourceID};

pub use diagnostic;
pub use lsp_types;

fn location_to_position(line_str: &str, line: usize, column: usize, byte_index: usize) -> Result<Position, DiagnosticError> {
    if column > line_str.len() {
        let max = line_str.len();
        let given = column;

        Err(DiagnosticError::ColumnTooLarge { given, max })
    }
    else if !line_str.is_char_boundary(column) {
        let given = byte_index;

        Err(DiagnosticError::InvalidCharBoundary { given })
    }
    else {
        let line_utf16 = line_str[..column].encode_utf16();
        let character = line_utf16.count() as u32;
        let line = line as u32;

        Ok(Position { line, character })
    }
}

pub fn byte_index_to_position(files: &SourceCache, file_id: &SourceID, byte_index: usize) -> Result<Position, DiagnosticError> {
    let source = files.get_text(file_id)?;

    let line_index = files.line_index(file_id, byte_index)?;
    let line_span = files.line_range(file_id, line_index).unwrap();

    let line_str = source.get(line_span.clone()).ok_or_else(|| DiagnosticError::IndexTooLarge {
        given: if line_span.start >= source.len() { line_span.start } else { line_span.end },
        max: source.len() - 1,
    })?;
    let column = byte_index - line_span.start;

    location_to_position(line_str, line_index, column, byte_index)
}

pub fn byte_span_to_range(files: &SourceCache, file_id: &SourceID, span: SourceID) -> Result<Range, DiagnosticError> {
    Ok(Range {
        start: byte_index_to_position(files, file_id, span.start)?,
        end: byte_index_to_position(files, file_id, span.end)?,
    })
}

fn character_to_line_offset(line: &str, character: u32) -> Result<usize, DiagnosticError> {
    let line_len = line.len();
    let mut character_offset = 0;

    let mut chars = line.chars();
    while let Some(ch) = chars.next() {
        if character_offset == character {
            let chars_off = chars.as_str().len();
            let ch_off = ch.len_utf8();

            return Ok(line_len - chars_off - ch_off);
        }

        character_offset += ch.len_utf16() as u32;
    }

    // Handle positions after the last character on the line
    if character_offset == character {
        Ok(line_len)
    }
    else {
        Err(DiagnosticError::ColumnTooLarge { given: character_offset as usize, max: line.len() })
    }
}

pub fn position_to_byte_index(files: &SourceCache, file_id: &SourceID, position: &Position) -> Result<usize, DiagnosticError> {
    let source = files.get_text(file_id)?;

    let line_span = files.line_range(file_id, position.line as usize).unwrap();
    let line_str = source.get(line_span.clone()).unwrap();

    let byte_offset = character_to_line_offset(line_str, position.character)?;

    Ok(line_span.start + byte_offset)
}

pub fn range_to_byte_span<F>(files: &SourceCache, file_id: &SourceID, range: &Range) -> Result<Span, DiagnosticError> {
    Ok(position_to_byte_index(files, file_id, &range.start)?..position_to_byte_index(files, file_id, &range.end)?)
}
