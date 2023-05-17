use std::vec::Vec;

use crate::{event::LogEvent, filter::Filter};

/// Receives structured log events.
pub trait LogSink: Send {
    /// Handle one event.
    fn emit(&mut self, event: &LogEvent);

    /// Flush buffered output when supported.
    fn flush(&mut self) {}
}

/// Wraps a sink with a level filter.
pub struct FilteredSink<S> {
    inner: S,
    filter: Filter,
}

impl<S: LogSink> FilteredSink<S> {
    /// Create a filtered sink.
    pub fn new(inner: S, filter: Filter) -> Self {
        Self { inner, filter }
    }
}

impl<S: LogSink> LogSink for FilteredSink<S> {
    fn emit(&mut self, event: &LogEvent) {
        if self.filter.accepts(event) {
            self.inner.emit(event);
        }
    }

    fn flush(&mut self) {
        self.inner.flush();
    }
}

/// Collects events into memory for tests.
#[derive(Clone, Debug, Default)]
pub struct VecSink {
    events: Vec<LogEvent>,
}

impl VecSink {
    /// Create an empty in-memory sink.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns collected events.
    pub fn events(&self) -> &[LogEvent] {
        &self.events
    }

    /// Clears collected events.
    pub fn clear(&mut self) {
        self.events.clear();
    }
}

impl LogSink for VecSink {
    fn emit(&mut self, event: &LogEvent) {
        self.events.push(event.clone());
    }
}
