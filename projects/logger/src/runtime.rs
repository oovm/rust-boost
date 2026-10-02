use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::vec::Vec;

use crate::event::LogEvent;
use crate::field::Field;
use crate::sink::{Filter, LogSink};
use crate::span::{current_parent_span_id, current_span_id};

static GLOBAL_SINK: OnceLock<Mutex<GlobalSinkState>> = OnceLock::new();
static GLOBAL_FILTER: OnceLock<Mutex<Filter>> = OnceLock::new();
static DROPPED_EVENTS: AtomicU64 = AtomicU64::new(0);

thread_local! {
    static SCOPE_FIELDS: RefCell<Vec<Field>> = const { RefCell::new(Vec::new()) };
}

struct GlobalSinkState {
    sink: Option<Arc<Mutex<Box<dyn LogSink>>>>,
}

impl GlobalSinkState {
    fn new() -> Self {
        Self { sink: None }
    }
}

fn global_state() -> &'static Mutex<GlobalSinkState> {
    GLOBAL_SINK.get_or_init(|| Mutex::new(GlobalSinkState::new()))
}

fn global_filter() -> &'static Mutex<Filter> {
    GLOBAL_FILTER.get_or_init(|| Mutex::new(Filter::new(crate::level::Level::Trace)))
}

fn record_dropped() {
    DROPPED_EVENTS.fetch_add(1, Ordering::Relaxed);
}

/// Returns the number of events dropped by filtering or missing sink.
pub fn dropped_events() -> u64 {
    DROPPED_EVENTS.load(Ordering::Relaxed)
}

/// Reset the dropped-event counter, mainly for tests.
pub fn reset_dropped_events() {
    DROPPED_EVENTS.store(0, Ordering::Relaxed);
}

/// Install the global root sink.
pub fn set_global_sink(sink: Box<dyn LogSink>) {
    let mut state = global_state().lock().expect("logger global sink mutex poisoned");
    state.sink = Some(Arc::new(Mutex::new(sink)));
}

/// Remove the installed global sink.
pub fn clear_global_sink() {
    let mut state = global_state().lock().expect("logger global sink mutex poisoned");
    state.sink = None;
}

/// Install the global minimum event level.
pub fn set_global_filter(filter: Filter) {
    let mut global = global_filter().lock().expect("logger global filter mutex poisoned");
    *global = filter;
}

/// Emit an event through the global facade.
pub fn emit(mut event: LogEvent) {
    event = enrich_event(event);
    let filter = global_filter().lock().expect("logger global filter mutex poisoned").clone();
    if !filter.accepts(&event) {
        record_dropped();
        return;
    }

    let sink = {
        let state = global_state().lock().expect("logger global sink mutex poisoned");
        state.sink.clone()
    };

    if let Some(sink) = sink {
        sink.lock().expect("logger sink mutex poisoned").emit(&event);
    } else {
        record_dropped();
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

fn enrich_event(event: LogEvent) -> LogEvent {
    let scope_fields = SCOPE_FIELDS.with(|scope| scope.borrow().clone());
    event.prepare_for_emit(current_span_id(), current_parent_span_id(), scope_fields)
}
