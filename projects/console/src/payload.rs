use std::string::String;
use std::vec::Vec;

use crate::field::FieldValue;

/// Structured message carried by diagnostic projections.
#[derive(Clone, Debug, PartialEq)]
pub struct MessagePayload {
    key: String,
    args: Vec<(String, FieldValue)>,
    fallback: Option<String>,
}

impl MessagePayload {
    /// Create a message payload from a stable key.
    pub fn new(key: impl Into<String>) -> Self {
        Self { key: key.into(), args: Vec::new(), fallback: None }
    }

    /// Attach a typed argument.
    pub fn with_arg(mut self, name: impl Into<String>, value: FieldValue) -> Self {
        self.args.push((name.into(), value));
        self
    }

    /// Attach optional fallback text.
    pub fn with_fallback(mut self, fallback: impl Into<String>) -> Self {
        self.fallback = Some(fallback.into());
        self
    }

    /// Returns the message key.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Returns typed arguments.
    pub fn args(&self) -> &[(String, FieldValue)] {
        &self.args
    }

    /// Returns optional fallback text.
    pub fn fallback(&self) -> Option<&str> {
        self.fallback.as_deref()
    }
}

/// Source reference carried by diagnostic projections.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceRefPayload {
    namespace: String,
    id: String,
    revision: Option<String>,
}

impl SourceRefPayload {
    /// Create a source reference payload.
    pub fn new(namespace: impl Into<String>, id: impl Into<String>) -> Self {
        Self { namespace: namespace.into(), id: id.into(), revision: None }
    }

    /// Attach a revision or snapshot identifier.
    pub fn with_revision(mut self, revision: impl Into<String>) -> Self {
        self.revision = Some(revision.into());
        self
    }

    /// Returns the namespace.
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Returns the identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the optional revision.
    pub fn revision(&self) -> Option<&str> {
        self.revision.as_deref()
    }
}

/// Half-open byte range carried by diagnostic projections.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ByteRangePayload {
    /// Inclusive start offset.
    pub start: u64,
    /// Exclusive end offset.
    pub end: u64,
}

/// Location payload used by diagnostic projections.
#[derive(Clone, Debug, PartialEq)]
pub enum LocationPayload {
    /// Text span in a referenced source.
    TextSpan {
        /// Referenced source identity.
        source: SourceRefPayload,
        /// Half-open byte range.
        range: ByteRangePayload,
    },
    /// Opaque tagged location for binary, IR, or future domains.
    Opaque {
        /// Location kind discriminator.
        kind: String,
        /// Structured fields for the location.
        fields: Vec<(String, FieldValue)>,
    },
}

/// Label payload used by diagnostic projections.
#[derive(Clone, Debug, PartialEq)]
pub struct LabelPayload {
    message: Option<MessagePayload>,
    location: LocationPayload,
    role: String,
}

impl LabelPayload {
    /// Create a label payload.
    pub fn new(location: LocationPayload, role: impl Into<String>) -> Self {
        Self { message: None, location, role: role.into() }
    }

    /// Attach an optional label message.
    pub fn with_message(mut self, message: MessagePayload) -> Self {
        self.message = Some(message);
        self
    }

    /// Returns the optional label message.
    pub fn message(&self) -> Option<&MessagePayload> {
        self.message.as_ref()
    }

    /// Returns the label location.
    pub fn location(&self) -> &LocationPayload {
        &self.location
    }

    /// Returns the label role.
    pub fn role(&self) -> &str {
        &self.role
    }
}

/// Action payload used by diagnostic projections.
#[derive(Clone, Debug, PartialEq)]
pub struct ActionPayload {
    id: String,
    args: Vec<(String, FieldValue)>,
}

impl ActionPayload {
    /// Create an action payload.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into(), args: Vec::new() }
    }

    /// Attach a typed argument.
    pub fn with_arg(mut self, name: impl Into<String>, value: FieldValue) -> Self {
        self.args.push((name.into(), value));
        self
    }

    /// Returns the action identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns typed arguments.
    pub fn args(&self) -> &[(String, FieldValue)] {
        &self.args
    }
}

/// Cause payload used by diagnostic projections.
#[derive(Clone, Debug, PartialEq)]
pub struct CausePayload {
    code: String,
    message: MessagePayload,
}

impl CausePayload {
    /// Create a cause payload.
    pub fn new(code: impl Into<String>, message: MessagePayload) -> Self {
        Self { code: code.into(), message }
    }

    /// Returns the cause code.
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Returns the cause message.
    pub fn message(&self) -> &MessagePayload {
        &self.message
    }
}

/// Structured diagnostic payload carried by `EventKind::Diagnostic`.
///
/// This type is owned by `console` so `diagnostic` can project into it without
/// reversing the dependency edge.
#[derive(Clone, Debug, PartialEq)]
pub struct DiagnosticPayload {
    code: String,
    severity: String,
    origin_stage: Option<String>,
    origin_tool: Option<String>,
    message: MessagePayload,
    primary: Option<LabelPayload>,
    secondary: Vec<LabelPayload>,
    notes: Vec<MessagePayload>,
    helps: Vec<MessagePayload>,
    cause: Option<CausePayload>,
    actions: Vec<ActionPayload>,
}

impl DiagnosticPayload {
    /// Create a diagnostic payload.
    pub fn new(code: impl Into<String>, severity: impl Into<String>, message: MessagePayload) -> Self {
        Self {
            code: code.into(),
            severity: severity.into(),
            origin_stage: None,
            origin_tool: None,
            message,
            primary: None,
            secondary: Vec::new(),
            notes: Vec::new(),
            helps: Vec::new(),
            cause: None,
            actions: Vec::new(),
        }
    }

    /// Attach producer stage metadata.
    pub fn with_origin_stage(mut self, stage: impl Into<String>) -> Self {
        self.origin_stage = Some(stage.into());
        self
    }

    /// Attach producer tool metadata.
    pub fn with_origin_tool(mut self, tool: impl Into<String>) -> Self {
        self.origin_tool = Some(tool.into());
        self
    }

    /// Attach the primary label.
    pub fn with_primary(mut self, label: LabelPayload) -> Self {
        self.primary = Some(label);
        self
    }

    /// Attach a secondary label.
    pub fn with_secondary(mut self, label: LabelPayload) -> Self {
        self.secondary.push(label);
        self
    }

    /// Attach a note message.
    pub fn with_note(mut self, note: MessagePayload) -> Self {
        self.notes.push(note);
        self
    }

    /// Attach a help message.
    pub fn with_help(mut self, help: MessagePayload) -> Self {
        self.helps.push(help);
        self
    }

    /// Attach a structured cause.
    pub fn with_cause(mut self, cause: CausePayload) -> Self {
        self.cause = Some(cause);
        self
    }

    /// Attach a structured action.
    pub fn with_action(mut self, action: ActionPayload) -> Self {
        self.actions.push(action);
        self
    }

    /// Returns the diagnostic code.
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Returns the severity string.
    pub fn severity(&self) -> &str {
        &self.severity
    }

    /// Returns the producer stage when present.
    pub fn origin_stage(&self) -> Option<&str> {
        self.origin_stage.as_deref()
    }

    /// Returns the producer tool when present.
    pub fn origin_tool(&self) -> Option<&str> {
        self.origin_tool.as_deref()
    }

    /// Returns the primary message.
    pub fn message(&self) -> &MessagePayload {
        &self.message
    }

    /// Returns the primary label when present.
    pub fn primary(&self) -> Option<&LabelPayload> {
        self.primary.as_ref()
    }

    /// Returns secondary labels.
    pub fn secondary(&self) -> &[LabelPayload] {
        &self.secondary
    }

    /// Returns note messages.
    pub fn notes(&self) -> &[MessagePayload] {
        &self.notes
    }

    /// Returns help messages.
    pub fn helps(&self) -> &[MessagePayload] {
        &self.helps
    }

    /// Returns the structured cause when present.
    pub fn cause(&self) -> Option<&CausePayload> {
        self.cause.as_ref()
    }

    /// Returns structured actions.
    pub fn actions(&self) -> &[ActionPayload] {
        &self.actions
    }
}
