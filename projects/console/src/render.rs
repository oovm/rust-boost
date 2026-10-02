use core::fmt::Write;

use logger::{EventKind, Level, LogEvent};

use crate::Paint;

/// Render one [`LogEvent`] as plain terminal output without styling.
pub fn render_plain(event: &LogEvent) -> String {
    format_event(event, false)
}

/// Render one [`LogEvent`] as human-visible terminal output.
///
/// Level and diagnostic severity are styled with [`Paint`]. When styling is disabled
/// through [`Paint::disable`], the output matches [`render_plain`].
pub fn render(event: &LogEvent) -> String {
    format_event(event, true)
}

fn format_event(event: &LogEvent, styled: bool) -> String {
    let level = format!("{}", event.level());
    let level = if styled { paint_level(event.level(), level) } else { level };
    let summary = if styled { paint_summary(event) } else { summary_text(event) };
    let mut out = String::new();
    let _ = write!(out, "[{}] {}: {}", level, event.target(), summary);
    out
}

fn paint_level(level: Level, text: String) -> String {
    let paint = match level {
        Level::Trace => Paint::new(text).dimmed(),
        Level::Debug => Paint::cyan(text),
        Level::Info => Paint::new(text),
        Level::Warn => Paint::yellow(text),
        Level::Error => Paint::red(text),
    };
    paint.to_string()
}

fn summary_text(event: &LogEvent) -> String {
    match event.kind() {
        EventKind::Diagnostic(payload) => format!(
            "diagnostic {} [{}] {}",
            payload.severity(),
            payload.code(),
            payload.message().key()
        ),
        EventKind::Log | EventKind::Progress | EventKind::Metric => event
            .fields()
            .iter()
            .find(|field| field.name() == "message")
            .map(|field| field.value().to_string())
            .unwrap_or_else(|| kind_summary(event.kind())),
    }
}

fn paint_summary(event: &LogEvent) -> String {
    match event.kind() {
        EventKind::Diagnostic(payload) => {
            paint_diagnostic_severity(payload.severity(), summary_text(event))
        }
        _ => summary_text(event),
    }
}

fn paint_diagnostic_severity(severity: &str, text: String) -> String {
    let paint = match severity {
        "info" => Paint::cyan(text),
        "warn" | "warning" => Paint::yellow(text),
        "bug" => Paint::magenta(text).bold(),
        _ => Paint::red(text),
    };
    paint.to_string()
}

fn kind_summary(kind: &EventKind) -> String {
    match kind {
        EventKind::Log => "log".to_string(),
        EventKind::Diagnostic(_) => "diagnostic".to_string(),
        EventKind::Progress => "progress".to_string(),
        EventKind::Metric => "metric".to_string(),
    }
}
