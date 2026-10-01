//! Legacy validation container for quick error call sites.

use crate::QError;

/// Legacy validation container retained for quick error call sites.
#[derive(Debug)]
pub enum Validation<T> {
    /// Parsed or validated value.
    Success(T),
    /// One or more quick errors.
    Failure(Vec<QError>),
}

impl<T> Validation<T> {
    /// Returns the success value when present.
    pub fn ok(self) -> Option<T> {
        match self {
            Self::Success(value) => Some(value),
            Self::Failure(_) => None,
        }
    }

    /// Returns collected errors when validation failed.
    pub fn errors(self) -> Option<Vec<QError>> {
        match self {
            Self::Success(_) => None,
            Self::Failure(errors) => Some(errors),
        }
    }
}
