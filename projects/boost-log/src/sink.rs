use std::vec::Vec;

use crate::{event::LogEvent, filter::Filter};

/// Result of asking a sink to flush buffered output.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlushOutcome {
    /// The sink wrote out everything it had buffered.
    Flushed,
    /// The sink could not write out its buffer.
    Failed,
    /// The sink no longer accepts work.
    Closed,
}

/// Receives structured log events.
pub trait LogSink: Send {
    /// Handle one event.
    fn emit(&mut self, event: &LogEvent);

    /// Flush buffered output and report the outcome.
    ///
    /// The default assumes the sink keeps no buffer.
    fn flush(&mut self) -> FlushOutcome {
        FlushOutcome::Flushed
    }
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

    fn flush(&mut self) -> FlushOutcome {
        self.inner.flush()
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
