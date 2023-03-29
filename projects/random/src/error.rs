use core::fmt::{self, Display, Formatter};

/// Stable failure classification of a `random` operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    /// The target has no implemented backend for the requested capability.
    Unsupported,
    /// The host entropy source exists but is currently unusable.
    EntropyUnavailable,
    /// The host entropy backend failed while producing bytes.
    BackendFailure,
    /// Range bounds are empty or reversed.
    InvalidRange,
    /// A rational probability has a zero denominator or an out-of-range numerator.
    InvalidProbability,
    /// A length or position cannot be represented by the profile.
    LengthOverflow,
    /// The stream reached the end of the `boost-chacha12-v1` capacity.
    StreamExhausted,
}

/// Failure returned by stream and source operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    pub(crate) fn new(kind: ErrorKind) -> Self {
        Self { kind }
    }

    /// Returns the stable failure classification.
    pub fn kind(self) -> ErrorKind {
        self.kind
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.kind {
            ErrorKind::Unsupported => f.write_str("the requested random capability is not supported by this target"),
            ErrorKind::EntropyUnavailable => f.write_str("host entropy is currently unavailable"),
            ErrorKind::BackendFailure => f.write_str("the host entropy backend failed"),
            ErrorKind::InvalidRange => f.write_str("the range bounds are empty or reversed"),
            ErrorKind::InvalidProbability => f.write_str("the rational probability is invalid"),
            ErrorKind::LengthOverflow => f.write_str("the length or position is outside the profile"),
            ErrorKind::StreamExhausted => f.write_str("the stream reached the end of its capacity"),
        }
    }
}

/// The `std` feature exposes the error through `std::error::Error`, which is the same trait as
/// `core::error::Error` in the standard library.
#[cfg(feature = "std")]
impl core::error::Error for Error {}
