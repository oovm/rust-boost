use std::sync::{Arc, Mutex};

use logger::{clear_global_sink, EventKind, Filter, Level, LogEvent, LogSink, VecSink, set_global_filter, set_global_sink};
use diagnostic::{emit_diagnostic, emit_diagnostic_set, DiagnosticSet};
use serial_test::serial;

use super::fixtures::text_diagnostic;

struct SharedSink(Arc<Mutex<VecSink>>);

impl LogSink for SharedSink {
    fn emit(&mut self, event: &LogEvent) {
        self.0.lock().expect("vec sink mutex poisoned").emit(event);
    }
}

#[test]
#[serial]
fn emit_diagnostic_posts_log_event() {
    clear_global_sink();
    let sink = Arc::new(Mutex::new(VecSink::new()));
    set_global_sink(Box::new(SharedSink(sink.clone())));
    set_global_filter(Filter::new(Level::Trace));

    emit_diagnostic(&text_diagnostic());

    let guard = sink.lock().expect("vec sink mutex poisoned");
    assert_eq!(guard.events().len(), 1);
    let payload = match guard.events()[0].kind() {
        EventKind::Diagnostic(payload) => payload,
        _ => panic!("expected diagnostic log event"),
    };
    assert_eq!(payload.code(), "oak.syntax.unexpected-token");
    assert_eq!(payload.severity(), "error");

    clear_global_sink();
}

#[test]
#[serial]
fn emit_diagnostic_set_posts_all_events() {
    clear_global_sink();
    let sink = Arc::new(Mutex::new(VecSink::new()));
    set_global_sink(Box::new(SharedSink(sink.clone())));
    set_global_filter(Filter::new(Level::Trace));

    let mut set = DiagnosticSet::new();
    set.push(text_diagnostic());
    set.push(text_diagnostic());
    emit_diagnostic_set(&set);

    let guard = sink.lock().expect("vec sink mutex poisoned");
    assert_eq!(guard.events().len(), 2);

    clear_global_sink();
}
