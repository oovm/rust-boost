//! Renderer-neutral projection helpers.

use crate::model::{Diagnostic, Message};

/// Returns the fallback text for a structured message when present.
pub fn message_fallback(message: &Message) -> &str {
    message.fallback().unwrap_or(message.key())
}

/// Returns renderer-neutral primary text for a diagnostic record.
pub fn diagnostic_message(diagnostic: &Diagnostic) -> &str {
    message_fallback(diagnostic.message())
}
