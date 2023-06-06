#![doc = include_str!("readme.md")]
#![deny(missing_docs)]

#[macro_use]
mod macros;

mod async_sink;
mod event;
mod field;
mod file;
mod filter;
mod json;
mod level;
mod payload;
mod runtime;
mod sink;
mod span;
mod task;

#[cfg(feature = "console")]
pub mod human;

pub use async_sink::{AsyncSink, AsyncSinkError};
pub use event::{EventKind, LogEvent, LogEventBuilder};
pub use field::{Field, FieldValue, Fields, IntoFieldValue};
pub use file::{FileSink, install_global_file_sink};
pub use filter::{Filter, FilterParseError};
pub use json::{JsonLinesSink, event_to_json};
pub use level::Level;
pub use payload::{
    ActionPayload, ByteRangePayload, CausePayload, DiagnosticPayload, LabelPayload, LocationPayload, MessagePayload, SourceRefPayload,
};
pub use runtime::{
    clear_global_sink, dropped_events, emit, flush_global_sink, reset_dropped_events, set_global_filter, set_global_sink, with_scope,
};
pub use sink::{FilteredSink, FlushOutcome, LogSink, VecSink};
pub use span::{LogSpan, SpanGuard, SpanId, current_span_id};
pub use task::{TaskContext, TaskId, TaskRef};

#[cfg(feature = "console")]
pub use human::{StderrFallbackSink, StderrSubscriberSink, install_global_subscriber, render, render_plain};

/// Enter a span and return a guard that exits it on drop.
pub fn enter(span: LogSpan) -> SpanGuard {
    SpanGuard::enter(span)
}

/// Exit a span guard explicitly.
pub fn exit(guard: SpanGuard) {
    drop(guard);
}
