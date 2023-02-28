use std::io::{self, Write};

use crate::event::{ConsoleEvent, EventKind};
use crate::field::{Field, FieldValue};
use crate::payload::{
    ActionPayload, CausePayload, DiagnosticPayload, LabelPayload, LocationPayload, MessagePayload,
};

/// Writes one JSON object per line for downstream log collectors.
pub struct JsonLinesSink<W> {
    writer: W,
}

impl<W> JsonLinesSink<W> {
    /// Create a JSON lines sink over `writer`.
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    /// Returns the inner writer.
    pub fn into_inner(self) -> W {
        self.writer
    }
}

impl<W: Write> JsonLinesSink<W> {
    /// Write one serialized event line.
    pub fn write_event(&mut self, event: &ConsoleEvent) -> io::Result<()> {
        writeln!(self.writer, "{}", event_to_json(event))
    }
}

impl<W: Write + Send> crate::sink::ConsoleSink for JsonLinesSink<W> {
    fn emit(&mut self, event: &ConsoleEvent) {
        let _ = self.write_event(event);
    }

    fn flush(&mut self) {
        let _ = self.writer.flush();
    }
}

/// Serialize one console event to a JSON object string.
pub fn event_to_json(event: &ConsoleEvent) -> String {
    let mut parts = Vec::new();
    parts.push(format!("\"kind\":{}", json_string(event_kind_name(event.kind()))));
    parts.push(format!("\"level\":{}", json_string(&event.level().to_string())));
    parts.push(format!("\"target\":{}", json_string(event.target())));
    parts.push(format!(
        "\"timestampSecs\":{}",
        event
            .timestamp()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0)
    ));
    if let Some(span_id) = event.span_id() {
        parts.push(format!("\"spanId\":{}", span_id.as_u64()));
    }
    if let Some(parent_span_id) = event.parent_span_id() {
        parts.push(format!("\"parentSpanId\":{}", parent_span_id.as_u64()));
    }
    if !event.fields().is_empty() {
        parts.push(format!("\"fields\":{}", fields_to_json(event.fields())));
    }
    if let EventKind::Diagnostic(payload) = event.kind() {
        parts.push(format!("\"diagnostic\":{}", diagnostic_payload_to_json(payload)));
    }
    format!("{{{}}}", parts.join(","))
}

fn event_kind_name(kind: &EventKind) -> &'static str {
    match kind {
        EventKind::Log => "log",
        EventKind::Diagnostic(_) => "diagnostic",
        EventKind::Progress => "progress",
        EventKind::Metric => "metric",
    }
}

fn fields_to_json(fields: &[Field]) -> String {
    let items = fields
        .iter()
        .map(|field| format!("{}:{}", json_string(field.name()), field_value_to_json(field.value())))
        .collect::<Vec<_>>();
    format!("{{{}}}", items.join(","))
}

fn field_value_to_json(value: &FieldValue) -> String {
    match value {
        FieldValue::Bool(value) => value.to_string(),
        FieldValue::I64(value) => value.to_string(),
        FieldValue::U64(value) => value.to_string(),
        FieldValue::F64(value) => value.to_string(),
        FieldValue::Str(value) => json_string(value),
        FieldValue::Debug(value) => json_string(value),
    }
}

fn diagnostic_payload_to_json(payload: &DiagnosticPayload) -> String {
    let mut parts = Vec::new();
    parts.push(format!("\"code\":{}", json_string(payload.code())));
    parts.push(format!("\"severity\":{}", json_string(payload.severity())));
    if let Some(stage) = payload.origin_stage() {
        parts.push(format!("\"originStage\":{}", json_string(stage)));
    }
    if let Some(tool) = payload.origin_tool() {
        parts.push(format!("\"originTool\":{}", json_string(tool)));
    }
    parts.push(format!("\"message\":{}", message_payload_to_json(payload.message())));
    if let Some(primary) = payload.primary() {
        parts.push(format!("\"primary\":{}", label_payload_to_json(primary)));
    }
    if !payload.secondary().is_empty() {
        let items = payload
            .secondary()
            .iter()
            .map(label_payload_to_json)
            .collect::<Vec<_>>();
        parts.push(format!("\"secondary\":[{}]", items.join(",")));
    }
    if !payload.notes().is_empty() {
        let items = payload.notes().iter().map(message_payload_to_json).collect::<Vec<_>>();
        parts.push(format!("\"notes\":[{}]", items.join(",")));
    }
    if !payload.helps().is_empty() {
        let items = payload.helps().iter().map(message_payload_to_json).collect::<Vec<_>>();
        parts.push(format!("\"helps\":[{}]", items.join(",")));
    }
    if let Some(cause) = payload.cause() {
        parts.push(format!("\"cause\":{}", cause_payload_to_json(cause)));
    }
    if !payload.actions().is_empty() {
        let items = payload.actions().iter().map(action_payload_to_json).collect::<Vec<_>>();
        parts.push(format!("\"actions\":[{}]", items.join(",")));
    }
    format!("{{{}}}", parts.join(","))
}

fn message_payload_to_json(message: &MessagePayload) -> String {
    let mut parts = vec![format!("\"key\":{}", json_string(message.key()))];
    if !message.args().is_empty() {
        let args = message
            .args()
            .iter()
            .map(|(name, value)| format!("{}:{}", json_string(name), field_value_to_json(value)))
            .collect::<Vec<_>>();
        parts.push(format!("\"args\":{{{}}}", args.join(",")));
    }
    if let Some(fallback) = message.fallback() {
        parts.push(format!("\"fallback\":{}", json_string(fallback)));
    }
    format!("{{{}}}", parts.join(","))
}

fn label_payload_to_json(label: &LabelPayload) -> String {
    let mut parts = vec![
        format!("\"role\":{}", json_string(label.role())),
        format!("\"location\":{}", location_payload_to_json(label.location())),
    ];
    if let Some(message) = label.message() {
        parts.push(format!("\"message\":{}", message_payload_to_json(message)));
    }
    format!("{{{}}}", parts.join(","))
}

fn location_payload_to_json(location: &LocationPayload) -> String {
    match location {
        LocationPayload::TextSpan { source, range } => {
            format!(
                "{{\"kind\":\"text\",\"source\":{{\"namespace\":{},\"id\":{}}},\"range\":{{\"start\":{},\"end\":{}}}}}",
                json_string(source.namespace()),
                json_string(source.id()),
                range.start,
                range.end
            )
        }
        LocationPayload::Opaque { kind, fields } => {
            format!(
                "{{\"kind\":{},\"fields\":{}}}",
                json_string(kind),
                fields_to_json(
                    &fields
                        .iter()
                        .map(|(name, value)| Field::new(name.clone(), value.clone()))
                        .collect::<Vec<_>>()
                )
            )
        }
    }
}

fn cause_payload_to_json(cause: &CausePayload) -> String {
    format!(
        "{{\"code\":{},\"message\":{}}}",
        json_string(cause.code()),
        message_payload_to_json(cause.message())
    )
}

fn action_payload_to_json(action: &ActionPayload) -> String {
    let mut parts = vec![format!("\"id\":{}", json_string(action.id()))];
    if !action.args().is_empty() {
        let args = action
            .args()
            .iter()
            .map(|(name, value)| format!("{}:{}", json_string(name), field_value_to_json(value)))
            .collect::<Vec<_>>();
        parts.push(format!("\"args\":{{{}}}", args.join(",")));
    }
    format!("{{{}}}", parts.join(","))
}

fn json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out.push('"');
    out
}
