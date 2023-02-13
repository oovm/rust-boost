use diagnostic::json::{DiagnosticEnvelope, SCHEMA_VERSION};

use super::fixtures::{
    binary_diagnostic, member_diagnostic, object_diagnostic, sample_set, semantic_diagnostic, text_diagnostic,
};

#[test]
fn wire_envelope_roundtrip() {
    let envelope = DiagnosticEnvelope::from_set(&sample_set());
    assert_eq!(envelope.schema_version, SCHEMA_VERSION);
    assert_eq!(envelope.diagnostics.len(), 5);

    let json = serde_json::to_string_pretty(&envelope).unwrap();
    let decoded: DiagnosticEnvelope = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.diagnostics.len(), 5);
    assert_eq!(decoded.diagnostics[0].code().as_str(), "oak.syntax.unexpected-token");
}

#[test]
fn binary_location_serializes() {
    let diagnostic = binary_diagnostic();
    let json = serde_json::to_value(&diagnostic).unwrap();
    assert_eq!(json["primary"]["location"]["kind"], "binary");
}

#[test]
#[ignore = "run manually to refresh on-disk JSON fixtures"]
fn write_wire_fixtures() {
    use std::fs;
    use std::path::PathBuf;

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    fs::create_dir_all(&root).unwrap();

    let write = |name: &str, value: &serde_json::Value| {
        let path = root.join(name);
        fs::write(path, serde_json::to_string_pretty(value).unwrap() + "\n").unwrap();
    };

    write("text.json", &serde_json::to_value(text_diagnostic()).unwrap());
    write("binary.json", &serde_json::to_value(binary_diagnostic()).unwrap());
    write("member.json", &serde_json::to_value(member_diagnostic()).unwrap());
    write("object.json", &serde_json::to_value(object_diagnostic()).unwrap());
    write("semantic.json", &serde_json::to_value(semantic_diagnostic()).unwrap());
    write(
        "envelope.json",
        &serde_json::to_value(DiagnosticEnvelope::from_set(&sample_set())).unwrap(),
    );
}
