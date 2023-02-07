use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::{self, Display, Formatter};

use super::location::DiagnosticLocation;

/// A structured message with stable key and typed arguments.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Message {
    key: String,
    args: Vec<(String, MessageArg)>,
    fallback: Option<String>,
}

impl Message {
    /// Create a message from a stable key.
    pub fn new(key: impl Into<String>) -> Self {
        Self { key: key.into(), args: Vec::new(), fallback: None }
    }

    /// Attach a typed argument.
    pub fn with_arg(mut self, name: impl Into<String>, value: MessageArg) -> Self {
        self.args.push((name.into(), value));
        self
    }

    /// Attach optional fallback text for renderers.
    pub fn with_fallback(mut self, fallback: impl Into<String>) -> Self {
        self.fallback = Some(fallback.into());
        self
    }

    /// Returns the message key.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Returns the typed arguments.
    pub fn args(&self) -> &[(String, MessageArg)] {
        &self.args
    }

    /// Returns optional fallback text.
    pub fn fallback(&self) -> Option<&str> {
        self.fallback.as_deref()
    }
}

/// Typed message parameter values allowed on the wire.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(tag = "kind", rename_all = "kebab-case"))]
pub enum MessageArg {
    /// Boolean value.
    Bool(bool),
    /// Signed integer.
    I64(i64),
    /// Unsigned integer serialized as decimal string on JSON wire.
    U64(u64),
    /// Text value.
    Text(String),
    /// Ordered list of text values.
    TextList(Vec<String>),
    /// Reference to another diagnostic location.
    Location(DiagnosticLocation),
}

impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(fallback) = &self.fallback {
            return f.write_str(fallback);
        }
        f.write_str(&self.key)
    }
}
