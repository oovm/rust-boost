use console::{
    event_to_json, ByteRangePayload, ConsoleEvent, ConsoleSink, DiagnosticPayload, FieldValue,
    JsonLinesSink, LabelPayload, Level, LocationPayload, MessagePayload, SourceRefPayload,
};

#[test]
fn event_to_json_serializes_log_event() {
    let event = ConsoleEvent::log(Level::Info, "worker")
        .field("message", FieldValue::str("ready"))
        .build();
    let json = event_to_json(&event);
    assert!(json.contains("\"kind\":\"log\""));
    assert!(json.contains("\"target\":\"worker\""));
    assert!(json.contains("\"message\":\"ready\""));
}

#[test]
fn event_to_json_serializes_diagnostic_payload() {
    let payload = DiagnosticPayload::new(
        "panduck.adapter.not-implemented",
        "error",
        MessagePayload::new("panduck.adapter.not-implemented").with_fallback("not implemented"),
    );
    let event = ConsoleEvent::diagnostic(payload);
    let json = event_to_json(&event);
    assert!(json.contains("\"kind\":\"diagnostic\""));
    assert!(json.contains("\"code\":\"panduck.adapter.not-implemented\""));
    assert!(json.contains("\"severity\":\"error\""));
    assert!(json.contains("\"fallback\":\"not implemented\""));
}

#[test]
fn event_to_json_serializes_text_span_revision() {
    let payload = DiagnosticPayload::new(
        "oak.syntax.unexpected-token",
        "error",
        MessagePayload::new("oak.syntax.unexpected-token"),
    )
        .with_primary(
            LabelPayload::new(
                LocationPayload::TextSpan {
                    source: SourceRefPayload::new("oak", "sample.xml").with_revision("1"),
                    range: ByteRangePayload { start: 12, end: 13 },
                },
                "primary",
            ),
        );
    let json = event_to_json(&ConsoleEvent::diagnostic(payload));
    assert!(json.contains("\"revision\":\"1\""));
}

#[test]
fn json_lines_sink_writes_one_line_per_event() {
    let mut buffer = Vec::new();
    let mut sink = JsonLinesSink::new(&mut buffer);
    sink.emit(
        &ConsoleEvent::log(Level::Warn, "cli")
            .field("message", FieldValue::str("retry"))
            .build(),
    );
    let text = String::from_utf8(buffer).expect("utf-8 output");
    assert_eq!(text.matches('\n').count(), 1);
    assert!(text.contains("\"kind\":\"log\""));
}

#[test]
fn json_string_escapes_quotes() {
    let event = ConsoleEvent::log(Level::Error, "test")
        .field("message", FieldValue::str("say \"hi\""))
        .build();
    let json = event_to_json(&event);
    assert!(json.contains(r#"say \"hi\""#));
}
