use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering};
use std::string::String;
use std::vec::Vec;

use crate::field::{Field, Fields};

static NEXT_SPAN_ID: AtomicU64 = AtomicU64::new(1);

thread_local! {
    static SPAN_STACK: RefCell<Vec<SpanId>> = const { RefCell::new(Vec::new()) };
}

/// Identifier for an active console span.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SpanId(u64);

impl SpanId {
    fn next() -> Self {
        Self(NEXT_SPAN_ID.fetch_add(1, Ordering::Relaxed))
    }

    /// Returns the raw span identifier.
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

/// Span metadata attached to emitted events.
#[derive(Clone, Debug, PartialEq)]
pub struct ConsoleSpan {
    id: SpanId,
    target: String,
    parent: Option<SpanId>,
    fields: Fields,
}

impl ConsoleSpan {
    /// Create a span with the given target.
    pub fn new(target: impl Into<String>) -> Self {
        Self {
            id: SpanId::next(),
            target: target.into(),
            parent: current_span_id(),
            fields: Fields::new(),
        }
    }

    /// Attach a field to the span.
    pub fn field(mut self, name: impl Into<String>, value: crate::field::FieldValue) -> Self {
        self.fields.push(name, value);
        self
    }

    /// Returns the span identifier.
    pub fn id(&self) -> SpanId {
        self.id
    }

    /// Returns the span target.
    pub fn target(&self) -> &str {
        &self.target
    }

    /// Returns the parent span identifier when present.
    pub fn parent(&self) -> Option<SpanId> {
        self.parent
    }

    /// Returns span fields.
    pub fn fields(&self) -> &[Field] {
        self.fields.as_slice()
    }
}

/// Guard that exits a span when dropped.
#[derive(Debug)]
pub struct SpanGuard {
    span_id: SpanId,
}

impl SpanGuard {
    /// Enter a span and return a guard that exits it on drop.
    pub fn enter(span: ConsoleSpan) -> Self {
        SPAN_STACK.with(|stack| stack.borrow_mut().push(span.id));
        Self { span_id: span.id }
    }

    /// Returns the entered span identifier.
    pub fn span_id(&self) -> SpanId {
        self.span_id
    }
}

impl Drop for SpanGuard {
    fn drop(&mut self) {
        SPAN_STACK.with(|stack| {
            let mut stack = stack.borrow_mut();
            if let Some(top) = stack.last() {
                if *top == self.span_id {
                    stack.pop();
                }
            }
        });
    }
}

/// Returns the current span identifier when inside a span.
pub fn current_span_id() -> Option<SpanId> {
    SPAN_STACK.with(|stack| stack.borrow().last().copied())
}

/// Returns the parent span identifier for the current span.
pub fn current_parent_span_id() -> Option<SpanId> {
    SPAN_STACK.with(|stack| {
        let stack = stack.borrow();
        if stack.len() >= 2 {
            stack.get(stack.len() - 2).copied()
        } else {
            None
        }
    })
}
