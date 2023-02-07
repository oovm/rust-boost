use core::fmt::{self, Display, Formatter};

/// A versioned, namespaced diagnostic code.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DiagnosticCode(String);

impl DiagnosticCode {
    /// Create a diagnostic code from a stable dotted identifier.
    pub fn new(code: impl Into<String>) -> Self {
        Self(code.into())
    }

    /// Returns the code string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for DiagnosticCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for DiagnosticCode {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}
