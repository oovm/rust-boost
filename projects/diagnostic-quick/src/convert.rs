//! Conversion from quick errors to structured diagnostics.

use diagnostic::{
    ByteRange, Diagnostic, DiagnosticCode, DiagnosticLabel, DiagnosticLocation, DiagnosticOrigin, LabelRole, Message,
    SourceRef,
};
use source_cache::SourceID;

use crate::{QError, QErrorKind};

/// Convert a quick error into a structured diagnostic record.
pub fn qerror_to_structured(error: &QError) -> Diagnostic {
    let (code, origin, severity, message, primary) = match &*error.error {
        QErrorKind::Syntax(syntax) => (
            DiagnosticCode::new("quick.syntax.error"),
            DiagnosticOrigin::new("quick", "syntax"),
            error.severity,
            Message::new("quick.syntax.error").with_fallback(syntax.message.clone()),
            syntax_label(syntax),
        ),
        QErrorKind::IO(io) => (
            DiagnosticCode::new("quick.io.error"),
            DiagnosticOrigin::new("quick", "io"),
            error.severity,
            Message::new("quick.io.error").with_fallback(io.message.clone()),
            None,
        ),
        QErrorKind::Runtime(runtime) => (
            DiagnosticCode::new("quick.runtime.error"),
            DiagnosticOrigin::new("quick", "runtime"),
            error.severity,
            Message::new("quick.runtime.error").with_fallback(runtime.message.clone()),
            None,
        ),
        QErrorKind::Custom(message) => (
            DiagnosticCode::new("quick.custom.error"),
            DiagnosticOrigin::new("quick", "custom"),
            error.severity,
            Message::new("quick.custom.error").with_fallback(message.clone()),
            None,
        ),
    };

    let mut diagnostic = Diagnostic::new(code, severity, origin, message);
    if let Some(label) = primary {
        diagnostic = diagnostic.with_primary(label);
    }
    diagnostic
}

fn syntax_label(syntax: &crate::SyntaxError) -> Option<DiagnosticLabel> {
    let range = ByteRange::new(syntax.span.start as u64, syntax.span.end as u64).ok()?;
    Some(DiagnosticLabel::new(
        DiagnosticLocation::Text {
            source: source_ref_for_id(&syntax.file),
            range,
        },
        Message::new("quick.syntax.span").with_fallback(syntax.message.clone()),
        LabelRole::Primary,
    ))
}

/// Build a stable source reference for a cached identifier.
pub fn source_ref_for_id(file: &SourceID) -> SourceRef {
    SourceRef::new("quick", format!("source:{file:?}"))
}
