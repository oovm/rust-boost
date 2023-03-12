#![doc = include_str!("../readme.md")]
#![deny(missing_docs)]

#[macro_use]
mod macros;

mod color;
mod draw;
mod paint;
mod sink;
mod style;
mod subscriber;

pub use color::Color;
pub use draw::{Background, Console, Foreground, Palette, StreamAwareFmt, StreamType};
pub use paint::Paint;
pub use sink::StderrFallbackSink;
pub use style::Style;
pub use subscriber::install_global_subscriber;

#[deprecated(
    since = "0.1.0",
    note = "use `logger` types and APIs instead of transitional `Console*` names"
)]
pub use logger::{
    ActionPayload, ByteRangePayload, CausePayload, DiagnosticPayload, EventKind, Field, FieldValue,
    Fields, FileSink, Filter, FilteredSink, JsonLinesSink, LabelPayload, Level, LocationPayload,
    LogEvent as ConsoleEvent, LogEventBuilder as ConsoleEventBuilder, LogSink as ConsoleSink,
    LogSpan as ConsoleSpan, MessagePayload, SourceRefPayload, SpanGuard, SpanId, VecSink,
    clear_global_sink, dropped_events, emit, enter, exit, reset_dropped_events, set_global_filter,
    set_global_sink, with_scope,
};
#[deprecated(
    since = "0.1.0",
    note = "use `logger::{current_span_id, event_to_json, install_global_file_sink}` instead"
)]
pub use logger::{current_span_id, event_to_json, install_global_file_sink};

/// Emit a structured console event through the shared logger facade.
#[deprecated(since = "0.1.0", note = "use `logger::event` instead")]
pub use logger::event;
/// Create a console span through the shared logger facade.
#[deprecated(since = "0.1.0", note = "use `logger::span` instead")]
pub use logger::span;
/// Convert supported literal types into [`FieldValue`].
#[deprecated(since = "0.1.0", note = "use `logger::field_value` instead")]
pub use logger::field_value;
