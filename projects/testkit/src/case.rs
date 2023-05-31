//! Stable identity of a parameterized case.
//!
//! A case is named by its declaration, never by an address, a thread, or the order in which cases
//! happen to run. This slice provides the validated identity; the parameter tables that generate
//! cases arrive with the procedural macro layer.

use core::fmt::{self, Display, Formatter};

/// A validated, stable case name.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CaseId(String);

/// A rejected case identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaseError {
    /// The case name is empty.
    Empty,
}

impl Display for CaseError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("a case name must not be empty"),
        }
    }
}

impl std::error::Error for CaseError {}

impl CaseId {
    /// Creates a case identity from a non-empty name.
    pub fn new(name: impl Into<String>) -> Result<Self, CaseError> {
        let name = name.into();
        if name.is_empty() { Err(CaseError::Empty) } else { Ok(Self(name)) }
    }

    /// Returns the case name.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for CaseId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl AsRef<str> for CaseId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
