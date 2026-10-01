use core::fmt::{self, Display, Formatter};

/// Severity of a diagnostic record.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum DiagnosticSeverity {
    /// Internal invariant or implementation error.
    Bug,
    /// Current object or stage cannot complete normally.
    Error,
    /// Work can continue, but a concrete problem exists.
    Warning,
    /// Information the user or tool should know.
    Info,
    /// Suggestive, non-blocking guidance.
    Hint,
}

impl Display for DiagnosticSeverity {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bug => f.write_str("bug"),
            Self::Error => f.write_str("error"),
            Self::Warning => f.write_str("warning"),
            Self::Info => f.write_str("info"),
            Self::Hint => f.write_str("hint"),
        }
    }
}
