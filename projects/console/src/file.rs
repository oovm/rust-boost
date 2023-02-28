use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use crate::event::ConsoleEvent;
use crate::json::JsonLinesSink;
use crate::sink::ConsoleSink;

/// Appends structured console events to a file as JSON lines.
pub struct FileSink {
    inner: JsonLinesSink<File>,
}

impl FileSink {
    /// Open `path` for append, creating the file when missing.
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Self { inner: JsonLinesSink::new(file) })
    }

    /// Create a sink over an already opened file handle.
    pub fn from_writer(file: File) -> Self {
        Self { inner: JsonLinesSink::new(file) }
    }

    /// Flush buffered output to disk.
    pub fn flush(&mut self) -> io::Result<()> {
        self.inner.flush();
        Ok(())
    }
}

impl ConsoleSink for FileSink {
    fn emit(&mut self, event: &ConsoleEvent) {
        self.inner.emit(event);
    }

    fn flush(&mut self) {
        self.inner.flush();
    }
}

/// Install a file-backed global sink, creating parent directories when needed.
pub fn install_global_file_sink(path: impl AsRef<Path>) -> io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let sink = FileSink::open(path)?;
    crate::set_global_sink(Box::new(sink));
    Ok(())
}
