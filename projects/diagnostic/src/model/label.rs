use super::{DiagnosticLocation, Message};

/// Role of a diagnostic label.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum LabelRole {
    /// Primary location for the diagnostic.
    Primary,
    /// Secondary related location.
    Secondary,
    /// Additional context location.
    Context,
}

/// A labelled diagnostic location with structured message text.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DiagnosticLabel {
    location: DiagnosticLocation,
    message: Message,
    role: LabelRole,
}

impl DiagnosticLabel {
    /// Create a diagnostic label.
    pub fn new(location: DiagnosticLocation, message: Message, role: LabelRole) -> Self {
        Self { location, message, role }
    }

    /// Returns the label location.
    pub fn location(&self) -> &DiagnosticLocation {
        &self.location
    }

    /// Returns the label message.
    pub fn message(&self) -> &Message {
        &self.message
    }

    /// Returns the label role.
    pub fn role(&self) -> LabelRole {
        self.role
    }
}
