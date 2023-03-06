#![doc = include_str!("../readme.md")]
#![deny(missing_docs)]

#[macro_use]
mod macros;

mod event;
mod field;
mod file;
mod json;
mod level;
mod payload;
mod runtime;
mod sink;
mod span;

pub use event::{EventKind, LogEvent, LogEventBuilder};
pub use field::{Field, FieldValue, Fields};
pub use file::{FileSink, install_global_file_sink};
pub use json::{JsonLinesSink, event_to_json};
pub use level::Level;
pub use payload::{
    ActionPayload, ByteRangePayload, CausePayload, DiagnosticPayload, LabelPayload, LocationPayload,
    MessagePayload, SourceRefPayload,
};
pub use runtime::{
    clear_global_sink, dropped_events, emit, reset_dropped_events, set_global_filter, set_global_sink,
    with_scope,
};
pub use sink::{Filter, FilteredSink, LogSink, VecSink};
pub use span::{LogSpan, SpanGuard, SpanId, current_span_id};

/// Enter a span and return a guard that exits it on drop.
pub fn enter(span: LogSpan) -> SpanGuard {
    SpanGuard::enter(span)
}

/// Exit a span guard explicitly.
pub fn exit(guard: SpanGuard) {
    drop(guard);
}
