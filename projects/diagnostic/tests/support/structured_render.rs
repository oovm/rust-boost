use diagnostic::terminal::{structured_to_terminal, SourceRegistry, Config};
use diagnostic::{
    ByteRange, Diagnostic, DiagnosticCode, DiagnosticLabel, DiagnosticLocation, DiagnosticOrigin, LabelRole, Message,
    SourceRef,
};
use diagnostic::DiagnosticSeverity;
use source_cache::SourceCache;

#[test]
fn structured_text_diagnostic_renders() {
    let mut cache = SourceCache::default();
    let source = cache.load_text(include_str!("../simple/sample.tao"), "sample.tao");
    let source_ref = SourceRef::new("oak", "sample.tao").unwrap();

    let mut registry = SourceRegistry::new();
    registry.register(&source_ref, source);

    let diagnostic = Diagnostic::new(
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
    ));

    let terminal = structured_to_terminal(&diagnostic, &registry, Config::default().with_color(false)).unwrap();
    terminal.print(&cache).unwrap();
}

#[test]
fn structured_member_diagnostic_falls_back_to_note() {
    use diagnostic::{MappingPrecision, MemberPath, MemberSegment};

    let registry = SourceRegistry::new();
    let diagnostic = Diagnostic::new(
        DiagnosticCode::new("acorn.container.need-range"),
        DiagnosticSeverity::Warning,
        DiagnosticOrigin::new("acorn", "zip"),
        Message::new("acorn.container.need-range").with_fallback("need range"),
    )
    .with_primary(DiagnosticLabel::new(
        DiagnosticLocation::Member {
            container: SourceRef::new("acorn", "docx.zip").unwrap(),
            member: MemberPath::new(vec![MemberSegment::new("zip", "word/document.xml")]),
            range: None,
            precision: MappingPrecision::Container,
        },
        Message::new("label.member").with_fallback("member"),
        LabelRole::Primary,
    ));

    assert!(structured_to_terminal(&diagnostic, &registry, Config::default().with_color(false)).is_ok());
}
