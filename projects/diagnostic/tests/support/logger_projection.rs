use logger::{event_to_json, EventKind, LocationPayload};

use super::fixtures::text_diagnostic;
use diagnostic::{diagnostic_to_log_event, diagnostic_to_payload};

#[test]
fn text_diagnostic_projects_to_log_event() {
    let diagnostic = text_diagnostic();
    let event = diagnostic_to_log_event(&diagnostic);
    let payload = match event.kind() {
        EventKind::Diagnostic(payload) => payload,
        _ => panic!("expected diagnostic event kind"),
    };

    assert_eq!(payload.code(), "oak.syntax.unexpected-token");
    assert_eq!(payload.severity(), "error");
    assert_eq!(payload.origin_stage(), Some("parse"));
    assert_eq!(payload.origin_tool(), Some("oak/xml"));
    assert_eq!(payload.message().key(), "oak.syntax.unexpected-token");

    let primary = payload.primary().expect("primary label");
    assert!(matches!(primary.location(), LocationPayload::TextSpan { .. }));
}

#[test]
fn text_diagnostic_log_event_json_matches_fixture() {
    let event = diagnostic_to_log_event(&text_diagnostic());
    let json = event_to_json(&event);
    let actual: serde_json::Value = serde_json::from_str(&json).expect("log event json");
    let expected: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/log-text-event.json")).expect("fixture json");

    assert_eq!(actual["kind"], expected["kind"]);
    assert_eq!(actual["level"], expected["level"]);
    assert_eq!(actual["target"], expected["target"]);
    assert_eq!(actual["diagnostic"], expected["diagnostic"]);
    assert!(actual.get("timestampSecs").is_some(), "log events must carry timestampSecs");
}

#[test]
fn diagnostic_to_payload_matches_log_event_payload() {
    let diagnostic = text_diagnostic();
    let event = diagnostic_to_log_event(&diagnostic);
    let payload = diagnostic_to_payload(&diagnostic);

    match event.kind() {
        EventKind::Diagnostic(event_payload) => {
            assert_eq!(event_payload.code(), payload.code());
            assert_eq!(event_payload.severity(), payload.severity());
        }
        _ => panic!("expected diagnostic event kind"),
    }
}
