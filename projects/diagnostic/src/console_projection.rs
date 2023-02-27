//! Project structured diagnostics into `console` events without reversing the dependency edge.

use console::{
    ActionPayload, ByteRangePayload, CausePayload, ConsoleEvent, DiagnosticPayload, FieldValue, LabelPayload,
    LocationPayload, MessagePayload, SourceRefPayload,
};

use crate::model::{
    Diagnostic, DiagnosticAction, DiagnosticCause, DiagnosticLabel, DiagnosticLocation, DiagnosticSeverity, Message,
    MessageArg, SourceRef,
};

/// Project a diagnostic record into a console event with `EventKind::Diagnostic`.
pub fn diagnostic_to_event(diagnostic: &Diagnostic) -> ConsoleEvent {
    ConsoleEvent::diagnostic(diagnostic_to_payload(diagnostic))
}

/// Project a diagnostic record into a console-owned payload.
pub fn diagnostic_to_payload(diagnostic: &Diagnostic) -> DiagnosticPayload {
    let mut payload = DiagnosticPayload::new(
        diagnostic.code().as_str(),
        severity_name(diagnostic.severity()),
        message_to_payload(diagnostic.message()),
    );

    let origin = diagnostic.origin();
    if let Some(stage) = origin.stage() {
        payload = payload.with_origin_stage(stage);
    }
    payload = payload.with_origin_tool(format!("{}/{}", origin.namespace(), origin.component()));

    if let Some(primary) = diagnostic.primary() {
        payload = payload.with_primary(label_to_payload(primary));
    }
    for label in diagnostic.secondary() {
        payload = payload.with_secondary(label_to_payload(label));
    }
    for note in diagnostic.notes() {
        payload = payload.with_note(message_to_payload(note));
    }
    for help in diagnostic.helps() {
        payload = payload.with_help(message_to_payload(help));
    }
    if let Some(cause) = diagnostic.cause() {
        payload = payload.with_cause(cause_to_payload(cause));
    }
    for action in diagnostic.actions() {
        payload = payload.with_action(action_to_payload(action));
    }

    payload
}

fn severity_name(severity: DiagnosticSeverity) -> String {
    severity.to_string()
}

fn message_to_payload(message: &Message) -> MessagePayload {
    let mut payload = MessagePayload::new(message.key());
    for (name, value) in message.args() {
        payload = payload.with_arg(name.clone(), message_arg_to_field(value));
    }
    if let Some(fallback) = message.fallback() {
        payload = payload.with_fallback(fallback);
    }
    payload
}

fn label_to_payload(label: &DiagnosticLabel) -> LabelPayload {
    let mut payload = LabelPayload::new(location_to_payload(label.location()), label_role_name(label.role()));
    if !label.message().key().is_empty() || !label.message().args().is_empty() || label.message().fallback().is_some() {
        payload = payload.with_message(message_to_payload(label.message()));
    }
    payload
}

fn label_role_name(role: crate::model::LabelRole) -> String {
    match role {
        crate::model::LabelRole::Primary => "primary",
        crate::model::LabelRole::Secondary => "secondary",
        crate::model::LabelRole::Context => "context",
    }
    .to_string()
}

fn cause_to_payload(cause: &DiagnosticCause) -> CausePayload {
    CausePayload::new(cause.code().as_str(), message_to_payload(cause.message()))
}

fn action_to_payload(action: &DiagnosticAction) -> ActionPayload {
    let mut payload = ActionPayload::new(action.id());
    for (name, value) in action.args() {
        payload = payload.with_arg(name.clone(), message_arg_to_field(value));
    }
    payload
}

fn source_ref_to_payload(source: &SourceRef) -> SourceRefPayload {
    let mut payload = SourceRefPayload::new(source.namespace(), source.id());
    if let Some(revision) = source.revision() {
        payload = payload.with_revision(revision);
    }
    payload
}

fn location_to_payload(location: &DiagnosticLocation) -> LocationPayload {
    match location {
        DiagnosticLocation::Text { source, range } => LocationPayload::TextSpan {
            source: source_ref_to_payload(source),
            range: ByteRangePayload { start: range.start, end: range.end },
        },
        other => LocationPayload::Opaque {
            kind: other.kind_str().to_string(),
            fields: location_opaque_fields(other),
        },
    }
}

fn location_opaque_fields(location: &DiagnosticLocation) -> Vec<(String, FieldValue)> {
    match location {
        DiagnosticLocation::Binary { source, address_space, range } => vec![
            ("source.namespace".into(), FieldValue::str(source.namespace())),
            ("source.id".into(), FieldValue::str(source.id())),
            ("address_space.namespace".into(), FieldValue::str(address_space.namespace())),
            ("address_space.id".into(), FieldValue::str(address_space.id())),
            ("range.start".into(), FieldValue::U64(range.start)),
            ("range.end".into(), FieldValue::U64(range.end)),
        ],
        DiagnosticLocation::Member { container, member, range, precision } => {
            let mut fields = vec![
                ("container.namespace".into(), FieldValue::str(container.namespace())),
                ("container.id".into(), FieldValue::str(container.id())),
                ("precision".into(), FieldValue::str(mapping_precision_name(*precision))),
            ];
            for (index, segment) in member.segments().iter().enumerate() {
                fields.push((format!("member.{index}.kind"), FieldValue::str(segment.kind())));
                fields.push((format!("member.{index}.name"), FieldValue::str(segment.name())));
            }
            if let Some(range) = range {
                fields.push(("range.start".into(), FieldValue::U64(range.start)));
                fields.push(("range.end".into(), FieldValue::U64(range.end)));
            }
            fields
        }
        DiagnosticLocation::Object { source, object } => vec![
            ("source.namespace".into(), FieldValue::str(source.namespace())),
            ("source.id".into(), FieldValue::str(source.id())),
            ("object.kind".into(), FieldValue::str(object.kind())),
            ("object.id".into(), FieldValue::str(object.id())),
        ],
        DiagnosticLocation::Semantic { document, path } => vec![
            ("document.namespace".into(), FieldValue::str(document.namespace())),
            ("document.id".into(), FieldValue::str(document.id())),
            ("path".into(), FieldValue::str(path.as_str())),
        ],
        DiagnosticLocation::Virtual { source, range } => vec![
            ("source.namespace".into(), FieldValue::str(source.namespace())),
            ("source.id".into(), FieldValue::str(source.id())),
            ("range.start".into(), FieldValue::U64(range.start)),
            ("range.end".into(), FieldValue::U64(range.end)),
        ],
        DiagnosticLocation::Text { .. } => Vec::new(),
    }
}

fn mapping_precision_name(precision: crate::model::MappingPrecision) -> &'static str {
    match precision {
        crate::model::MappingPrecision::Exact => "exact",
        crate::model::MappingPrecision::Container => "container",
        crate::model::MappingPrecision::Unknown => "unknown",
    }
}

fn message_arg_to_field(value: &MessageArg) -> FieldValue {
    match value {
        MessageArg::Bool(value) => FieldValue::Bool(*value),
        MessageArg::I64(value) => FieldValue::I64(*value),
        MessageArg::U64(value) => FieldValue::U64(*value),
        MessageArg::Text(value) => FieldValue::str(value.clone()),
        MessageArg::TextList(values) => FieldValue::debug(values),
        MessageArg::Location(location) => FieldValue::debug(location_to_payload(location)),
    }
}
