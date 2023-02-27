#![doc = include_str!("../readme.md")]
#![deny(missing_docs)]

#[macro_use]
mod macros;

mod color;
mod draw;
mod event;
mod field;
mod level;
mod paint;
mod payload;
mod runtime;
mod sink;
mod span;
mod style;

pub use color::Color;
pub use draw::{Background, Console, Foreground, Palette, StreamAwareFmt, StreamType};
pub use event::{ConsoleEvent, ConsoleEventBuilder, EventKind};
pub use field::{Field, FieldValue, Fields};
pub use level::Level;
pub use paint::Paint;
pub use payload::{
    ActionPayload, ByteRangePayload, CausePayload, DiagnosticPayload, LabelPayload, LocationPayload,
    MessagePayload, SourceRefPayload,
};
pub use runtime::{clear_global_sink, emit, set_global_filter, set_global_sink, with_scope};
pub use sink::{ConsoleSink, Filter, FilteredSink, StderrFallbackSink, VecSink};
pub use span::{ConsoleSpan, SpanGuard, SpanId, current_span_id};
pub use style::Style;

/// Enter a span and return a guard that exits it on drop.
pub fn enter(span: ConsoleSpan) -> SpanGuard {
    SpanGuard::enter(span)
}

/// Exit a span guard explicitly.
pub fn exit(guard: SpanGuard) {
    drop(guard);
}
