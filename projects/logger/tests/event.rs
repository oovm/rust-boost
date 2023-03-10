use std::sync::{Arc, Mutex};

use logger::{
    DiagnosticPayload, EventKind, Field, FieldValue, Filter, Level, LogEvent, LogSink, MessagePayload,
    VecSink, clear_global_sink, current_span_id, dropped_events, enter, event, reset_dropped_events,
    set_global_filter, set_global_sink, span, with_scope,
};
use serial_test::serial;

struct SharedSink(Arc<Mutex<VecSink>>);

impl LogSink for SharedSink {
    fn emit(&mut self, event: &LogEvent) {
        self.0.lock().expect("vec sink mutex poisoned").emit(event);
    }
}

#[test]
#[serial]
fn event_macro_emits_structured_log() {
    clear_global_sink();
    reset_dropped_events();
    let sink = Arc::new(Mutex::new(VecSink::new()));
    set_global_sink(Box::new(SharedSink(sink.clone())));
    set_global_filter(Filter::new(Level::Trace));

    event!(Info, "test.target", message = "hello");

    let guard = sink.lock().expect("vec sink mutex poisoned");
    assert_eq!(guard.events().len(), 1);
    assert_eq!(guard.events()[0].target(), "test.target");
    assert_eq!(guard.events()[0].level(), Level::Info);
    assert_eq!(dropped_events(), 0);
    clear_global_sink();
}

#[test]
#[serial]
fn global_emit_applies_scope_fields() {
    clear_global_sink();
    reset_dropped_events();
    let sink = Arc::new(Mutex::new(VecSink::new()));
    set_global_sink(Box::new(SharedSink(sink.clone())));
    set_global_filter(Filter::new(Level::Trace));

    let scope = [Field::new("request_id", FieldValue::str("abc"))];
    with_scope(&scope, || {
        let _guard = enter(span!("worker", step = 1));
        event!(Warn, "worker", message = "retrying");
    });

    let guard = sink.lock().expect("vec sink mutex poisoned");
    assert_eq!(guard.events().len(), 1);
    assert!(guard.events()[0].fields().iter().any(|field| field.name() == "request_id"));
    assert!(guard.events()[0].span_id().is_some());
    clear_global_sink();
}

#[test]
fn filter_drops_low_priority_events() {
    let filter = Filter::new(Level::Warn);
    let info = LogEvent::log(Level::Info, "quiet").build();
    let error = LogEvent::log(Level::Error, "loud").build();
    assert!(!filter.accepts(&info));
    assert!(filter.accepts(&error));
}

#[test]
#[serial]
fn emit_without_sink_is_silent_and_counts_drop() {
    clear_global_sink();
    reset_dropped_events();
    set_global_filter(Filter::new(Level::Trace));

    event!(Error, "orphan", message = "nowhere");

    assert_eq!(dropped_events(), 1);
}

#[test]
#[serial]
fn filtered_emit_increments_drop_counter() {
    clear_global_sink();
    reset_dropped_events();
    let sink = Arc::new(Mutex::new(VecSink::new()));
    set_global_sink(Box::new(SharedSink(sink.clone())));
    set_global_filter(Filter::new(Level::Error));

    event!(Info, "quiet", message = "ignored");
    event!(Error, "loud", message = "kept");

    let guard = sink.lock().expect("vec sink mutex poisoned");
    assert_eq!(guard.events().len(), 1);
    assert_eq!(dropped_events(), 1);
    clear_global_sink();
}

#[test]
fn span_guard_exits_on_drop() {
    let _guard = enter(span!("task"));
    assert!(current_span_id().is_some());
    drop(_guard);
    assert!(current_span_id().is_none());
}

#[test]
fn diagnostic_event_preserves_kind() {
    let payload = DiagnosticPayload::new("oak.syntax.unexpected-token", "error", MessagePayload::new("oak.unexpected"));
    let event = LogEvent::diagnostic(payload);
    assert!(matches!(event.kind(), EventKind::Diagnostic(_)));
    assert_eq!(event.level(), Level::Error);
}

struct ReentrantSink;

impl LogSink for ReentrantSink {
    fn emit(&mut self, _event: &LogEvent) {
        set_global_sink(Box::new(VecSink::new()));
    }
}

#[test]
#[serial]
fn sink_emit_does_not_hold_global_registration_lock() {
    clear_global_sink();
    reset_dropped_events();
    set_global_sink(Box::new(ReentrantSink));
    set_global_filter(Filter::new(Level::Trace));

    event!(Info, "reentrant", message = "swap sink during emit");

    clear_global_sink();
}
