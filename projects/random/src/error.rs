use core::fmt::{self, Display, Formatter};

/// Stable failure classification of a `random` operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RandomErrorKind {
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
    /// A distribution parameter model is internally inconsistent or unusable.
    InvalidParameter,
    /// A sampled value cannot be expressed by the numerical contract of its distribution.
    NumericalFailure,
    /// A length or position cannot be represented by the profile.
    LengthOverflow,
    /// The stream reached the end of the `boost-chacha12-v1` capacity.
    StreamExhausted,
}

/// Failure returned by stream, source, and entropy operations.
///
/// The [`RandomErrorKind`] classification is stable. A raw platform code may be carried next to it for
/// reporting, but no platform message string is part of the contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RandomError {
    kind: RandomErrorKind,
    platform_code: Option<i32>,
}

impl RandomError {
    /// Creates an error from a stable classification.
    pub const fn new(kind: RandomErrorKind) -> Self {
        Self { kind, platform_code: None }
    }

    /// Creates an error that also preserves the raw code reported by a platform backend.
    pub const fn with_platform_code(kind: RandomErrorKind, platform_code: i32) -> Self {
        Self { kind, platform_code: Some(platform_code) }
    }

    /// Returns the stable failure classification.
    pub const fn kind(self) -> RandomErrorKind {
        self.kind
    }

    /// Returns the raw platform code when the failure came from a platform backend.
    pub const fn platform_code(self) -> Option<i32> {
        self.platform_code
    }
}

impl Display for RandomError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.kind {
            RandomErrorKind::Unsupported => f.write_str("the requested random capability is not supported by this target"),
            RandomErrorKind::EntropyUnavailable => f.write_str("host entropy is currently unavailable"),
            RandomErrorKind::BackendFailure => f.write_str("the host entropy backend failed"),
            RandomErrorKind::InvalidRange => f.write_str("the range bounds are empty or reversed"),
            RandomErrorKind::InvalidProbability => f.write_str("the rational probability is invalid"),
            RandomErrorKind::InvalidParameter => f.write_str("the distribution parameters are invalid"),
            RandomErrorKind::NumericalFailure => f.write_str("the sampled value is outside the numerical contract"),
            RandomErrorKind::LengthOverflow => f.write_str("the length or position is outside the profile"),
            RandomErrorKind::StreamExhausted => f.write_str("the stream reached the end of its capacity"),
        }
    }
}

/// The `std` feature exposes the error through `std::error::Error`, which is the same trait as
/// `core::error::Error` in the standard library.
#[cfg(feature = "std")]
impl core::error::Error for RandomError {}
