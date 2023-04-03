use diagnostic::terminal::{structured_to_terminal, Config, SourceRegistry};
use diagnostic::{
    ByteRange, Diagnostic, DiagnosticCode, DiagnosticLabel, DiagnosticLocation, DiagnosticOrigin, LabelRole, Message,
    SourceRef,
};
use diagnostic::DiagnosticSeverity;
use diagnostic::terminal::SourceCache;

fn sample_text_diagnostic(source_ref: SourceRef) -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::new("oak.syntax.unexpected-token"),
        DiagnosticSeverity::Error,
        DiagnosticOrigin::new("oak", "xml"),
        Message::new("oak.syntax.unexpected-token").with_fallback("unexpected token"),
    )
    .with_primary(DiagnosticLabel::new(
        DiagnosticLocation::Text {
            source: source_ref,
            range: ByteRange::new(32, 33).unwrap(),
        },
        Message::new("label.token").with_fallback("token"),
        LabelRole::Primary,
    ))
}

/// Source text of the golden fixture, with line endings normalized to LF.
///
/// The repository stores the fixture with LF, while a Windows checkout may materialize CRLF. The
/// label range is an absolute byte range, so the checkout's line endings would otherwise change the
/// reported column and the rendered golden on different machines. The recorded golden is compared
/// through the same normalization for the same reason.
fn fixture_text() -> String {
    include_str!("../simple/sample.tao").replace("\r\n", "\n")
}

fn render_terminal_golden(diagnostic: &Diagnostic, cache: &SourceCache, registry: &SourceRegistry) -> String {
    let terminal = structured_to_terminal(diagnostic, registry, Config::default().with_color(false)).unwrap();
    let mut buffer = Vec::new();
    terminal.write(cache, &mut buffer).unwrap();
    String::from_utf8(buffer).expect("terminal output must be utf-8")
}

#[test]
fn terminal_text_diagnostic_matches_golden() {
    let mut cache = SourceCache::default();
    let text = fixture_text();
    let source = cache.load_text(text.as_str(), "sample.tao");
    let source_ref = SourceRef::new("oak", "sample.tao").unwrap();

    let mut registry = SourceRegistry::new();
    registry.register(&source_ref, source);

    let actual = render_terminal_golden(&sample_text_diagnostic(source_ref), &cache, &registry);
    let expected = include_str!("../golden/terminal-text.golden").replace("\r\n", "\n");
    assert_eq!(actual, expected);
}

#[test]
#[ignore = "run manually to refresh terminal golden fixtures"]
fn write_terminal_golden_fixture() {
    use std::fs;
    use std::path::PathBuf;

    let mut cache = SourceCache::default();
    let text = fixture_text();
    let source = cache.load_text(text.as_str(), "sample.tao");
    let source_ref = SourceRef::new("oak", "sample.tao").unwrap();

    let mut registry = SourceRegistry::new();
    registry.register(&source_ref, source);

    let rendered = render_terminal_golden(&sample_text_diagnostic(source_ref), &cache, &registry);
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden/terminal-text.golden");
    fs::write(path, rendered).expect("write terminal golden");
}
