use std::string::String;
use std::time::SystemTime;
use std::vec::Vec;

use crate::field::{Field, FieldValue, Fields};
use crate::level::Level;
use crate::payload::DiagnosticPayload;
use crate::span::SpanId;

/// High-level event category.
#[derive(Clone, Debug, PartialEq)]
pub enum EventKind {
    /// Generic structured log event.
    Log,
    /// Structured diagnostic projection.
    Diagnostic(DiagnosticPayload),
    /// Progress update.
    Progress,
    /// Metric sample.
    Metric,
}

/// Structured console event emitted through the global facade.
#[derive(Clone, Debug, PartialEq)]
pub struct ConsoleEvent {
    kind: EventKind,
    level: Level,
    target: String,
    fields: Vec<Field>,
    timestamp: SystemTime,
    span_id: Option<SpanId>,
    parent_span_id: Option<SpanId>,
}

impl ConsoleEvent {
    /// Create a builder for a generic log event.
    pub fn log(level: Level, target: impl Into<String>) -> ConsoleEventBuilder {
        ConsoleEventBuilder::new(EventKind::Log, level, target)
    }

    /// Create a diagnostic event from a structured payload.
    pub fn diagnostic(payload: DiagnosticPayload) -> Self {
        let level = diagnostic_level(payload.severity());
        ConsoleEventBuilder::new(EventKind::Diagnostic(payload), level, "diagnostic").build()
    }

    /// Returns the event kind.
    pub fn kind(&self) -> &EventKind {
        &self.kind
    }

    /// Returns the event level.
    pub fn level(&self) -> Level {
        self.level
    }

    /// Returns the event target.
    pub fn target(&self) -> &str {
        &self.target
    }

    /// Returns attached fields.
    pub fn fields(&self) -> &[Field] {
        &self.fields
    }

    /// Returns the event timestamp.
    pub fn timestamp(&self) -> SystemTime {
        self.timestamp
    }

    /// Returns the active span id when present.
    pub fn span_id(&self) -> Option<SpanId> {
        self.span_id
    }

    /// Returns the parent span id when present.
    pub fn parent_span_id(&self) -> Option<SpanId> {
        self.parent_span_id
    }

    pub(crate) fn prepare_for_emit(
        mut self,
        span_id: Option<SpanId>,
        parent_span_id: Option<SpanId>,
        extra_fields: Vec<Field>,
    ) -> Self {
        if self.span_id.is_none() {
            self.span_id = span_id;
            self.parent_span_id = parent_span_id;
        }
        if !extra_fields.is_empty() {
            self.fields.extend(extra_fields);
        }
        self
    }
}

/// Builder for [`ConsoleEvent`].
#[derive(Clone, Debug)]
pub struct ConsoleEventBuilder {
    kind: EventKind,
    level: Level,
    target: String,
    fields: Fields,
}

impl ConsoleEventBuilder {
    /// Create a new event builder.
    pub fn new(kind: EventKind, level: Level, target: impl Into<String>) -> Self {
        Self { kind, level, target: target.into(), fields: Fields::new() }
    }

    /// Attach a field.
    pub fn field(mut self, name: impl Into<String>, value: FieldValue) -> Self {
        self.fields.push(name, value);
        self
    }

    /// Attach multiple fields.
    pub fn fields(mut self, fields: Fields) -> Self {
        for field in fields.into_vec() {
            self.fields.push(field.name().to_string(), field.value().clone());
        }
        self
    }

    /// Build the event with current span context filled in by the runtime.
    pub fn build(self) -> ConsoleEvent {
        ConsoleEvent {
            kind: self.kind,
            level: self.level,
            target: self.target,
            fields: self.fields.into_vec(),
            timestamp: SystemTime::now(),
            span_id: None,
            parent_span_id: None,
        }
    }
}

fn diagnostic_level(severity: &str) -> Level {
    match severity {
        "bug" | "error" => Level::Error,
        "warning" => Level::Warn,
        "info" => Level::Info,
        "hint" => Level::Debug,
        _ => Level::Info,
    }
}
