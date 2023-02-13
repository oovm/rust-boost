use diagnostic::json::{DiagnosticEnvelope, SCHEMA_VERSION};
use diagnostic::Diagnostic;

#[test]
fn text_fixture_roundtrip() {
    let json = include_str!("../fixtures/text.json");
    let diagnostic: Diagnostic = serde_json::from_str(json).unwrap();
    assert_eq!(diagnostic.code().as_str(), "oak.syntax.unexpected-token");
    assert_eq!(diagnostic.primary().unwrap().location().kind_str(), "text");
}

#[test]
fn binary_fixture_roundtrip() {
    let json = include_str!("../fixtures/binary.json");
    let diagnostic: Diagnostic = serde_json::from_str(json).unwrap();
    assert_eq!(diagnostic.primary().unwrap().location().kind_str(), "binary");
}

#[test]
fn member_fixture_roundtrip() {
    let json = include_str!("../fixtures/member.json");
    let diagnostic: Diagnostic = serde_json::from_str(json).unwrap();
    assert_eq!(diagnostic.primary().unwrap().location().kind_str(), "member");
}

#[test]
fn object_fixture_roundtrip() {
    let json = include_str!("../fixtures/object.json");
    let diagnostic: Diagnostic = serde_json::from_str(json).unwrap();
    assert_eq!(diagnostic.primary().unwrap().location().kind_str(), "object");
}

#[test]
fn semantic_fixture_roundtrip() {
    let json = include_str!("../fixtures/semantic.json");
    let diagnostic: Diagnostic = serde_json::from_str(json).unwrap();
    assert_eq!(diagnostic.primary().unwrap().location().kind_str(), "semantic");
}

#[test]
fn envelope_fixture_roundtrip() {
    let json = include_str!("../fixtures/envelope.json");
    let envelope: DiagnosticEnvelope = serde_json::from_str(json).unwrap();
    assert_eq!(envelope.schema_version, SCHEMA_VERSION);
    assert_eq!(envelope.diagnostics.len(), 5);
    assert!(!envelope.truncated);
    assert_eq!(envelope.dropped, 0);
}
