use alloc::string::String;

use super::DiagnosticCode;

/// A structured diagnostic action declaration.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DiagnosticAction {
    id: String,
    args: Vec<(String, super::MessageArg)>,
}

impl DiagnosticAction {
    /// Create a diagnostic action identifier with typed arguments.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into(), args: Vec::new() }
    }

    /// Attach a typed argument.
    pub fn with_arg(mut self, name: impl Into<String>, value: super::MessageArg) -> Self {
        self.args.push((name.into(), value));
        self
    }

    /// Returns the action identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the typed arguments.
    pub fn args(&self) -> &[(String, super::MessageArg)] {
        &self.args
    }
}

/// Structured cause chain entry.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DiagnosticCause {
    code: DiagnosticCode,
    message: super::Message,
}

impl DiagnosticCause {
    /// Create a structured cause entry.
    pub fn new(code: DiagnosticCode, message: super::Message) -> Self {
        Self { code, message }
    }

    /// Returns the cause code.
    pub fn code(&self) -> &DiagnosticCode {
        &self.code
    }

    /// Returns the cause message.
    pub fn message(&self) -> &super::Message {
        &self.message
    }
}
