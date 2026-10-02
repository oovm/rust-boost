use logger::{DiagnosticPayload, FieldValue, Level, LogEvent, MessagePayload};
use serial_test::serial;

use console::{Paint, render, render_plain};

#[test]
fn render_plain_log_event() {
    let event = LogEvent::log(Level::Info, "worker")
        .field("message", FieldValue::str("ready"))
        .build();
    assert_eq!(render_plain(&event), "[info] worker: ready");
}

#[test]
fn render_plain_diagnostic_event() {
    let payload = DiagnosticPayload::new(
        "oak.syntax.unexpected-token",
        "error",
        MessagePayload::new("oak.unexpected"),
    );
    let event = LogEvent::diagnostic(payload);
    assert_eq!(
        render_plain(&event),
        "[error] diagnostic: diagnostic error [oak.syntax.unexpected-token] oak.unexpected"
    );
}

#[test]
#[serial]
fn styled_render_applies_level_color() {
    Paint::enable();
    let event = LogEvent::log(Level::Error, "worker")
        .field("message", FieldValue::str("failed"))
        .build();
    let styled = render(&event);
    assert!(styled.contains("\x1B[31m"), "error level should be red");
    assert!(styled.contains("failed"));
}

#[test]
#[serial]
fn styled_render_matches_plain_when_disabled() {
    let event = LogEvent::log(Level::Warn, "worker")
        .field("message", FieldValue::str("retry"))
        .build();
    Paint::disable();
    assert_eq!(render(&event), render_plain(&event));
    Paint::enable();
}