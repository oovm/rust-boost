use std::io::{self, Write};
use std::vec::Vec;

use crate::event::{ConsoleEvent, EventKind};
use crate::level::Level;

/// Receives structured console events.
pub trait ConsoleSink: Send {
    /// Handle one event.
    fn emit(&mut self, event: &ConsoleEvent);

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
    pub fn accepts(&self, event: &ConsoleEvent) -> bool {
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

impl<S: ConsoleSink> FilteredSink<S> {
    /// Create a filtered sink.
    pub fn new(inner: S, filter: Filter) -> Self {
        Self { inner, filter }
    }
}

impl<S: ConsoleSink> ConsoleSink for FilteredSink<S> {
    fn emit(&mut self, event: &ConsoleEvent) {
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
    events: Vec<ConsoleEvent>,
}

impl VecSink {
    /// Create an empty in-memory sink.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns collected events.
    pub fn events(&self) -> &[ConsoleEvent] {
        &self.events
    }

    /// Clears collected events.
    pub fn clear(&mut self) {
        self.events.clear();
    }
}

impl ConsoleSink for VecSink {
    fn emit(&mut self, event: &ConsoleEvent) {
        self.events.push(event.clone());
    }
}

/// Minimal stderr fallback used when no global sink is installed.
pub struct StderrFallbackSink;

impl ConsoleSink for StderrFallbackSink {
    fn emit(&mut self, event: &ConsoleEvent) {
        let summary = match event.kind() {
            EventKind::Diagnostic(payload) => {
                format!(
                    "diagnostic {} [{}] {}",
                    payload.severity(),
                    payload.code(),
                    payload.message().key()
                )
            }
            EventKind::Log | EventKind::Progress | EventKind::Metric => {
                let message = event
                    .fields()
                    .iter()
                    .find(|field| field.name() == "message")
                    .map(|field| field.value().to_string())
                    .unwrap_or_else(|| event.kind().summary());
                format!("{} {}", event.level(), message)
            }
        };
        let _ = writeln!(io::stderr(), "[{}] {}: {}", event.level(), event.target(), summary);
    }
}

impl EventKind {
    fn summary(&self) -> String {
        match self {
            Self::Log => "log".to_string(),
            Self::Diagnostic(_) => "diagnostic".to_string(),
            Self::Progress => "progress".to_string(),
            Self::Metric => "metric".to_string(),
        }
    }
}
