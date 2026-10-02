use crate::access::{AccessError, ByteAccess};
use crate::range::ByteRange;

/// Zero-based line and UTF-8 byte column coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextCoords {
    /// Zero-based line index.
    pub line: u32,
    /// Zero-based UTF-8 byte column within the line.
    pub column: u32,
}

/// Line-start offsets for a UTF-8 byte buffer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineIndex {
    offsets: Vec<u64>,
}

impl LineIndex {
    /// Build a line index from raw bytes without trimming whitespace.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut offsets = Vec::new();
        offsets.push(0);

        let mut index = 0usize;
        while index < bytes.len() {
            let byte = bytes[index];
            if byte == b'\n' {
                index += 1;
                if index < bytes.len() {
                    offsets.push(index as u64);
                }
                continue;
            }
            if byte == b'\r' {
                let next = index + 1;
                if next < bytes.len() && bytes[next] == b'\n' {
                    index = next + 1;
                }
                else {
                    index += 1;
                }
                if index < bytes.len() {
                    offsets.push(index as u64);
                }
                continue;
            }
            index += 1;
        }

        if offsets.is_empty() {
            offsets.push(0);
        }

        Self { offsets }
    }

    /// Returns the number of lines.
    pub fn line_count(&self) -> usize {
        self.offsets.len()
    }

    /// Returns the start offset of a line.
    pub fn line_offset(&self, line: usize) -> Option<u64> {
        self.offsets.get(line).copied()
    }

    /// Returns all line start offsets.
    pub fn offsets(&self) -> &[u64] {
        &self.offsets
    }

    /// Map a byte offset to `(line, column)`.
    pub fn coords_at_offset(&self, bytes_len: u64, offset: u64) -> Result<TextCoords, AccessError> {
        if offset > bytes_len {
            return Err(AccessError::OutOfRange {
                len: bytes_len,
                range: ByteRange::new(offset, offset).expect("zero-length range is valid"),
            });
        }

        let line = match self.offsets.binary_search(&offset) {
            Ok(index) => index,
            Err(index) => index.saturating_sub(1),
        };
        let line_offset = self.offsets[line];
        let column = offset - line_offset;
        let column = u32::try_from(column).map_err(|_| AccessError::OutOfRange {
            len: bytes_len,
            range: ByteRange::new(offset, offset).expect("zero-length range is valid"),
        })?;
        let line = u32::try_from(line).map_err(|_| AccessError::OutOfRange {
            len: bytes_len,
            range: ByteRange::new(offset, offset).expect("zero-length range is valid"),
        })?;
        Ok(TextCoords { line, column })
    }
}

/// UTF-8 text view over a byte-accessing snapshot.
pub struct TextView<'a, T: ByteAccess + ?Sized> {
    access: &'a T,
    lines: &'a LineIndex,
}

impl<'a, T: ByteAccess + ?Sized> TextView<'a, T> {
    /// Create a text view over `access` using a pre-built line index.
    pub fn new(access: &'a T, lines: &'a LineIndex) -> Self {
        Self { access, lines }
    }

    /// Returns the line index.
    pub fn line_index(&self) -> &LineIndex {
        self.lines
    }

    /// Returns the number of lines.
    pub fn line_count(&self) -> usize {
        self.lines.line_count()
    }

    /// Returns the raw bytes for a line without trimming whitespace.
    pub fn line_at(&self, line: usize) -> Result<&[u8], AccessError> {
        let start = self.lines.line_offset(line).ok_or(AccessError::OutOfRange {
            len: self.access.len(),
            range: ByteRange::new(0, 0).expect("zero-length range is valid"),
        })?;
        let end = if line + 1 < self.lines.line_count() {
            self.lines.line_offset(line + 1).expect("line exists")
        }
        else {
            self.access.len()
        };
        let slice = self.access.read(ByteRange::new(start, end).expect("line bounds are ordered"))?;
        Ok(strip_line_ending(slice))
    }

    /// Map a byte offset to line and column coordinates.
    pub fn coords_at_offset(&self, offset: u64) -> Result<TextCoords, AccessError> {
        self.lines.coords_at_offset(self.access.len(), offset)
    }

    /// Decode a UTF-8 byte range.
    pub fn utf8_range(&self, range: ByteRange) -> Result<&str, AccessError> {
        let bytes = self.access.read(range)?;
        core::str::from_utf8(bytes).map_err(|error| AccessError::NotUtf8 {
            offset: range.start() + error.valid_up_to() as u64,
        })
    }
}

fn strip_line_ending(slice: &[u8]) -> &[u8] {
    if slice.ends_with(b"\r\n") {
        &slice[..slice.len() - 2]
    }
    else if slice.ends_with(b"\n") || slice.ends_with(b"\r") {
        &slice[..slice.len() - 1]
    }
    else {
        slice
    }
}
