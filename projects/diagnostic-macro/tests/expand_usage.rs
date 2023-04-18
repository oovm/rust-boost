use diagnostic::DiagnosticSeverity;
use diagnostic_macro::diagnose;

pub fn sample() -> Result<(), String> {
    let record = diagnose! {
        code: "macro.test.sample",
        severity: error,
        message: "sample message",
    };

    if record.code().as_str() != "macro.test.sample" {
        return Err("unexpected code".into());
    }
    if record.severity() != DiagnosticSeverity::Error {
        return Err("unexpected severity".into());
    }
    if record.origin().to_string() != "macro/diagnostic-macro" {
        return Err("unexpected origin".into());
    }
    if record.message().fallback() != Some("sample message") {
        return Err("unexpected message".into());
    }
    Ok(())
}
