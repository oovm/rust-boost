use core::fmt::{self, Display, Formatter};

/// Event importance used by filters, sinks, and fallback routing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Level {
    /// Fine-grained trace output.
    Trace,
    /// Developer-oriented debug output.
    Debug,
    /// Normal operational information.
    Info,
    /// Recoverable problems or suspicious conditions.
    Warn,
    /// Failures and invariant violations.
    Error,
}

impl Level {
    /// Returns whether this level is at least as severe as `other`.
    pub fn is_at_least(self, other: Self) -> bool {
        self >= other
    }
}

impl Display for Level {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Trace => f.write_str("trace"),
            Self::Debug => f.write_str("debug"),
            Self::Info => f.write_str("info"),
            Self::Warn => f.write_str("warn"),
            Self::Error => f.write_str("error"),
        }
    }
}
