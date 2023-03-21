//! Terminal source storage backed by `source::MemoryStore`.

use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::ops::Range;
use std::path::{Path, PathBuf};

use source::{
    AccessError, ByteAccess, IdentityError, MemorySnapshot, MemoryStore, SnapshotHandle, SnapshotRef, SourceRef,
};

/// Process-local source handle for terminal rendering.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct SourceID(SnapshotHandle);

impl Default for SourceID {
    fn default() -> Self {
        Self(SnapshotHandle::INVALID)
    }
}

impl SourceID {
    /// Attach a byte span to this source handle.
    pub fn with_range(self, range: Range<u32>) -> SourceSpan {
        SourceSpan { start: range.start, end: range.end, file: self }
    }
}

/// Byte span inside a terminal source.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct SourceSpan {
    /// Inclusive start offset.
    pub start: u32,
    /// Exclusive end offset.
    pub end: u32,
    /// Owning source handle.
    pub file: SourceID,
}

impl SourceSpan {
    /// Byte length of the span.
    pub fn length(&self) -> u32 {
        self.end.saturating_sub(self.start)
    }

    /// Returns whether `offset` lies inside the span.
    pub fn contains(&self, offset: u32) -> bool {
        self.start <= offset && offset < self.end
    }
}

/// Display path for a terminal source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourcePath {
    /// Anonymous source.
    Anonymous,
    /// Named snippet.
    Snippet(String),
    /// Local filesystem path.
    Local(PathBuf),
}

impl Display for SourcePath {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Anonymous => f.write_str("<anonymous>"),
            Self::Snippet(name) => f.write_str(name),
            Self::Local(path) => f.write_str(&path.display().to_string()),
        }
    }
}

/// One source line exposed to the legacy terminal renderer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceLine {
    /// Byte offset in the original source.
    pub offset: u32,
    /// Byte length of the line content.
    pub length: u32,
    /// Line text without trailing line endings.
    pub text: String,
}

impl SourceLine {
    /// Returns the half-open byte range of this line.
    pub fn range(&self) -> Range<u32> {
        self.offset..self.offset + self.length
    }

    /// Iterate characters in the line.
    pub fn chars(&self) -> impl Iterator<Item = char> + '_ {
        self.text.chars()
    }
}

/// In-memory source table for terminal diagnostics.
#[derive(Clone, Debug, Default)]
pub struct SourceCache {
    store: MemoryStore,
    paths: HashMap<u32, SourcePath>,
}

impl SourceCache {
    /// Load UTF-8 text from the filesystem.
    pub fn load_local<P>(&mut self, path: P) -> Result<SourceID, std::io::Error>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let bytes = std::fs::read(path)?;
        let display = path.to_string_lossy().into_owned();
        let id = self
            .insert_snapshot(bytes, SourcePath::Local(path.to_path_buf()), "file", display)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error.to_string()))?;
        Ok(id)
    }

    /// Load in-memory text under a display name.
    pub fn load_text<T, N>(&mut self, text: T, name: N) -> SourceID
    where
        T: AsRef<str>,
        N: AsRef<str>,
    {
        let name = name.as_ref().to_string();
        self.insert_snapshot(text.as_ref().as_bytes().to_vec(), SourcePath::Snippet(name.clone()), "terminal", name)
            .expect("terminal snippet identity is valid")
    }

    /// Resolve a source handle to a borrowed view.
    pub fn fetch(&self, file: &SourceID) -> Result<SourceView<'_>, std::io::Error> {
        match self.store.get(file.0) {
            Ok(snapshot) => Ok(SourceView { snapshot }),
            Err(AccessError::SnapshotReleased) => Err(std::io::Error::new(std::io::ErrorKind::NotFound, "source handle released")),
            Err(error) => Err(std::io::Error::other(error.to_string())),
        }
    }

    /// Returns the display path for a source handle.
    pub fn source_path(&self, file: &SourceID) -> Option<&SourcePath> {
        self.paths.get(&file.0.index())
    }
}

impl SourceCache {
    fn insert_snapshot(
        &mut self,
        bytes: Vec<u8>,
        path: SourcePath,
        namespace: &str,
        id: String,
    ) -> Result<SourceID, IdentityError> {
        let reference = SnapshotRef::new(SourceRef::new(namespace, id)?);
        let snapshot = MemorySnapshot::from_bytes(reference, bytes);
        let handle = self.store.insert(snapshot);
        self.paths.insert(handle.index(), path);
        Ok(SourceID(handle))
    }
}

/// Borrowed UTF-8 source view for terminal rendering.
pub struct SourceView<'a> {
    snapshot: &'a MemorySnapshot,
}

impl SourceView<'_> {
    /// Returns the number of lines.
    pub fn line_count(&self) -> usize {
        self.snapshot.text_view().line_count()
    }

    /// Returns all lines.
    pub fn lines(&self) -> Vec<SourceLine> {
        let count = self.line_count();
        (0..count).filter_map(|index| self.get_line(index)).collect()
    }

    /// Returns total known byte length.
    pub fn get_length(&self) -> usize {
        ByteAccess::len(self.snapshot) as usize
    }

    /// Returns a line by index.
    pub fn get_line(&self, idx: usize) -> Option<SourceLine> {
        let view = self.snapshot.text_view();
        let start = view.line_index().line_offset(idx)? as u32;
        let bytes = view.line_at(idx).ok()?;
        let text = std::str::from_utf8(bytes).map(|value| value.to_string()).ok()?;
        Some(SourceLine { offset: start, length: text.len() as u32, text })
    }

    /// Map a byte offset to line index and column.
    pub fn get_offset_line(&self, offset: u32) -> Option<(SourceLine, usize, u32)> {
        let coords = match self.snapshot.text_view().coords_at_offset(offset as u64) {
            Ok(coords) => coords,
            Err(_) => return None,
        };
        let line = self.get_line(coords.line as usize)?;
        Some((line, coords.line as usize, coords.column))
    }

    /// Returns the line range spanned by a byte range.
    pub fn get_line_range(&self, span: &Range<u32>) -> Range<usize> {
        let start = self.get_offset_line(span.start).map_or(0, |(_, line, _)| line);
        let end = self
            .get_offset_line(span.end.saturating_sub(1).max(span.start))
            .map_or(self.line_count(), |(_, line, _)| line + 1);
        start..end
    }
}
