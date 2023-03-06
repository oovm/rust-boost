use std::vec::Vec;

use crate::event::LogEvent;
use crate::level::Level;

/// Receives structured log events.
pub trait LogSink: Send {
    /// Handle one event.
    fn emit(&mut self, event: &LogEvent);

    /// Flush buffered output when supported.
    fn flush(&mut self) {
    }
}

/// Level-based event filter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Filter {
    min_level: Level,
}

impl Filter {
    /// Create a filter that accepts events at or above `min_level`.
    pub fn new(min_level: Level) -> Self {
        Self { min_level }
    }

    /// Returns whether the event should be accepted.
    pub fn accepts(&self, event: &LogEvent) -> bool {
        event.level().is_at_least(self.min_level)
    }

    /// Returns the configured minimum level.
    pub fn min_level(&self) -> Level {
        self.min_level
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
