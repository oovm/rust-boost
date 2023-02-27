use std::cell::RefCell;
use std::sync::{Mutex, OnceLock};
use std::vec::Vec;

use crate::event::ConsoleEvent;
use crate::field::Field;
use crate::level::Level;
use crate::sink::{ConsoleSink, Filter, StderrFallbackSink};
use crate::span::{current_parent_span_id, current_span_id};

static GLOBAL_SINK: OnceLock<Mutex<GlobalSinkState>> = OnceLock::new();
static GLOBAL_FILTER: OnceLock<Mutex<Filter>> = OnceLock::new();

thread_local! {
    static SCOPE_FIELDS: RefCell<Vec<Field>> = const { RefCell::new(Vec::new()) };
}

struct GlobalSinkState {
    sink: Option<Box<dyn ConsoleSink>>,
    fallback: StderrFallbackSink,
}

impl GlobalSinkState {
    fn new() -> Self {
        Self { sink: None, fallback: StderrFallbackSink }
    }
}

fn global_state() -> &'static Mutex<GlobalSinkState> {
    GLOBAL_SINK.get_or_init(|| Mutex::new(GlobalSinkState::new()))
}

fn global_filter() -> &'static Mutex<Filter> {
    GLOBAL_FILTER.get_or_init(|| Mutex::new(Filter::new(Level::Trace)))
}

/// Install the global root sink.
pub fn set_global_sink(sink: Box<dyn ConsoleSink>) {
    let mut state = global_state().lock().expect("console global sink mutex poisoned");
    state.sink = Some(sink);
}

/// Remove the installed global sink.
pub fn clear_global_sink() {
    let mut state = global_state().lock().expect("console global sink mutex poisoned");
    state.sink = None;
}

/// Install the global minimum event level.
pub fn set_global_filter(filter: Filter) {
    let mut global = global_filter().lock().expect("console global filter mutex poisoned");
    *global = filter;
}

/// Emit an event through the global facade.
pub fn emit(mut event: ConsoleEvent) {
    event = enrich_event(event);
    let filter = global_filter().lock().expect("console global filter mutex poisoned").clone();
    if !filter.accepts(&event) {
        return;
    }

    let mut state = global_state().lock().expect("console global sink mutex poisoned");
    if let Some(sink) = state.sink.as_mut() {
        sink.emit(&event);
    } else if event.level().is_at_least(Level::Warn) {
        state.fallback.emit(&event);
    }
}

/// Run a closure with additional scope fields attached to emitted events.
pub fn with_scope<R>(fields: &[Field], f: impl FnOnce() -> R) -> R {
    SCOPE_FIELDS.with(|scope| scope.borrow_mut().extend(fields.iter().cloned()));
    let result = f();
    SCOPE_FIELDS.with(|scope| {
        let mut scope = scope.borrow_mut();
        let trim = fields.len().min(scope.len());
        let new_len = scope.len() - trim;
        scope.truncate(new_len);
    });
    result
}

fn enrich_event(event: ConsoleEvent) -> ConsoleEvent {
    let scope_fields = SCOPE_FIELDS.with(|scope| scope.borrow().clone());
    event.prepare_for_emit(current_span_id(), current_parent_span_id(), scope_fields)
}
