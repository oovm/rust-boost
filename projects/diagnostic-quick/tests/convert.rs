use diagnostic::{DiagnosticSeverity};
use diagnostic_quick::{qerror_to_structured, QError};
use source_cache::SourceID;

#[test]
fn qerror_converts_to_structured_syntax_diagnostic() {
    let mut span = SourceID::default().with_range(4..8);
    let error = QError::syntax_error("unexpected token")
        .with_file(&span.file)
        .with_span(span);

    let diagnostic = qerror_to_structured(&error);
    assert_eq!(diagnostic.code().as_str(), "quick.syntax.error");
    assert_eq!(diagnostic.severity(), DiagnosticSeverity::Error);
    assert!(diagnostic.primary().is_some());
}

#[test]
fn runtime_error_converts_without_primary_label() {
    let error = QError::runtime_error("boom");
    let diagnostic = qerror_to_structured(&error);
    assert_eq!(diagnostic.code().as_str(), "quick.runtime.error");
    assert!(diagnostic.primary().is_none());
}
