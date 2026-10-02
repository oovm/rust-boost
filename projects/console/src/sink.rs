use std::io::{self, Write};

use logger::{EventKind, LogEvent, LogSink};

/// Minimal stderr fallback for console subscribers when no styled sink is installed.
pub struct StderrFallbackSink;

impl LogSink for StderrFallbackSink {
    fn emit(&mut self, event: &LogEvent) {
        let summary = match event.kind() {
            EventKind::Diagnostic(payload) => {
                format!(
                    "diagnostic {} [{}] {}",
                    payload.severity(),
                    payload.code(),
                    payload.message().key()
                )
            }
            EventKind::Log | EventKind::Progress | EventKind::Metric => {
                let message = event
                    .fields()
                    .iter()
                    .find(|field| field.name() == "message")
                    .map(|field| field.value().to_string())
                    .unwrap_or_else(|| kind_summary(event.kind()));
                format!("{} {}", event.level(), message)
            }
        };
        let _ = writeln!(io::stderr(), "[{}] {}: {}", event.level(), event.target(), summary);
    }
}

fn kind_summary(kind: &EventKind) -> String {
    match kind {
        EventKind::Log => "log".to_string(),
        EventKind::Diagnostic(_) => "diagnostic".to_string(),
        EventKind::Progress => "progress".to_string(),
        EventKind::Metric => "metric".to_string(),
    }
}
