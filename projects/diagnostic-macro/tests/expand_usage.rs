use diagnostic::{Diagnostic, DiagnosticCode, DiagnosticOrigin, DiagnosticSeverity, Message};

pub fn sample() -> Result<(), String> {
    let record = Diagnostic::new(
        DiagnosticCode::new("macro.test.sample"),
        DiagnosticSeverity::Error,
        DiagnosticOrigin::new("macro", "diagnostic-macro"),
        Message::new("macro.test.sample").with_fallback("sample message"),
    );

    if record.code().as_str() != "macro.test.sample" {
        return Err("unexpected code".into());
    }
    if record.severity() != DiagnosticSeverity::Error {
        return Err("unexpected severity".into());
    }
    if record.message().fallback() != Some("sample message") {
        return Err("unexpected message".into());
    }
    Ok(())
}
