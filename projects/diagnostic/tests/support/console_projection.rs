use console::{EventKind, LocationPayload};

use super::fixtures::{binary_diagnostic, member_diagnostic, text_diagnostic};
use diagnostic::{diagnostic_to_event, diagnostic_to_payload};

#[test]
fn text_diagnostic_projects_to_console_event() {
    let diagnostic = text_diagnostic();
    let event = diagnostic_to_event(&diagnostic);
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
fn binary_diagnostic_projects_opaque_location_fields() {
    let payload = diagnostic_to_payload(&binary_diagnostic());
    let primary = payload.primary().expect("primary label");
    match primary.location() {
        LocationPayload::Opaque { kind, fields } => {
            assert_eq!(kind, "binary");
            assert!(fields.iter().any(|(name, _)| name == "range.start"));
        }
        other => panic!("expected opaque binary location, got {other:?}"),
    }
}

#[test]
fn member_diagnostic_preserves_precision_field() {
    let payload = diagnostic_to_payload(&member_diagnostic());
    let primary = payload.primary().expect("primary label");
    match primary.location() {
        LocationPayload::Opaque { kind, fields } => {
            assert_eq!(kind, "member");
            assert!(fields.iter().any(|(name, value)| name == "precision" && value.to_string() == "container"));
        }
        other => panic!("expected opaque member location, got {other:?}"),
    }
}
