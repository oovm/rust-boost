use diagnostic::DiagnosticSeverity;

fn main() {
    let record = diagnostic_macro::diagnose! {
        code: "macro.example.sample",
        severity: error,
        message: "example diagnostic",
    };

    assert_eq!(record.code().as_str(), "macro.example.sample");
    assert_eq!(record.severity(), DiagnosticSeverity::Error);
    println!("{}", record.message().fallback().unwrap_or(record.message().key()));
}
