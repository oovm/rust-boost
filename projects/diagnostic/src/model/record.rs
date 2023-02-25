use alloc::vec::Vec;

use super::{
    DiagnosticAction, DiagnosticCode, DiagnosticLabel, DiagnosticOrigin, DiagnosticSeverity, Message,
};
use super::action::DiagnosticCause;

/// Structured diagnostic record.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Diagnostic {
    code: DiagnosticCode,
    severity: DiagnosticSeverity,
    origin: DiagnosticOrigin,
    message: Message,
    primary: Option<DiagnosticLabel>,
    secondary: Vec<DiagnosticLabel>,
    notes: Vec<Message>,
    helps: Vec<Message>,
    cause: Option<DiagnosticCause>,
    actions: Vec<DiagnosticAction>,
}

impl Diagnostic {
    /// Create a diagnostic record.
    pub fn new(
        code: DiagnosticCode,
        severity: DiagnosticSeverity,
        origin: DiagnosticOrigin,
        message: Message,
    ) -> Self {
        Self {
            code,
            severity,
            origin,
            message,
            primary: None,
            secondary: Vec::new(),
            notes: Vec::new(),
            helps: Vec::new(),
            cause: None,
            actions: Vec::new(),
        }
    }

    /// Set the primary label.
    pub fn with_primary(mut self, label: DiagnosticLabel) -> Self {
        self.primary = Some(label);
        self
    }

    /// Add a secondary label.
    pub fn with_secondary(mut self, label: DiagnosticLabel) -> Self {
        self.secondary.push(label);
        self
    }

    /// Add a note message.
    pub fn with_note(mut self, note: Message) -> Self {
        self.notes.push(note);
        self
    }

    /// Add a help message.
    pub fn with_help(mut self, help: Message) -> Self {
        self.helps.push(help);
        self
    }

    /// Attach a structured cause.
    pub fn with_cause(mut self, cause: DiagnosticCause) -> Self {
        self.cause = Some(cause);
        self
    }

    /// Attach a structured action.
    pub fn with_action(mut self, action: DiagnosticAction) -> Self {
        self.actions.push(action);
        self
    }

    /// Returns the diagnostic code.
    pub fn code(&self) -> &DiagnosticCode {
        &self.code
    }

    /// Returns the severity.
    pub fn severity(&self) -> DiagnosticSeverity {
        self.severity
    }

    /// Returns the producer origin.
    pub fn origin(&self) -> &DiagnosticOrigin {
        &self.origin
    }

    /// Returns the primary message.
    pub fn message(&self) -> &Message {
        &self.message
    }

    /// Returns the primary label when present.
    pub fn primary(&self) -> Option<&DiagnosticLabel> {
        self.primary.as_ref()
    }

    /// Returns secondary labels.
    pub fn secondary(&self) -> &[DiagnosticLabel] {
        &self.secondary
    }

    /// Returns note messages.
    pub fn notes(&self) -> &[Message] {
        &self.notes
    }

    /// Returns help messages.
    pub fn helps(&self) -> &[Message] {
        &self.helps
    }

    /// Returns the structured cause when present.
    pub fn cause(&self) -> Option<&DiagnosticCause> {
        self.cause.as_ref()
    }

    /// Returns structured actions.
    pub fn actions(&self) -> &[DiagnosticAction] {
        &self.actions
    }
}
