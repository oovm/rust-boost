use core::fmt::{self, Display, Formatter};

/// Failure of a Web host entropy bridge.
///
/// The variants are the distinctions this crate can really observe from the host rather than a copy
/// of a JavaScript error taxonomy: the `crypto` property is either usable or absent, and a request is
/// either served or refused. No platform code is invented for a thrown JavaScript value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WebError {
    /// The host object has no usable `crypto` property.
    Probe,
    /// The host refused the entropy request.
    Rejected,
}

/// Failure of a host entropy backend.
///
/// Only this layer handles the differences between operating systems, WASI, and Web hosts. A failure
/// that carries a platform code keeps the domain of that code, because [`Errno`](Self::Errno),
/// [`NtStatus`](Self::NtStatus), and [`Wasi`](Self::Wasi) are not one domain-free integer. A failure
/// without a raw code uses its own variant instead of inventing a sentinel code.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EntropyError {
    /// The target has no implemented backend for the request.
    Unsupported,
    /// The host source exists but is currently unusable.
    Unavailable,
    /// The backend failed without reporting a platform code.
    Failed,
    /// The backend reported a POSIX `errno`.
    Errno(i32),
    /// The backend reported an `NTSTATUS`.
    NtStatus(i32),
    /// The backend reported a WASI `errno`.
    Wasi(u16),
    /// The Web host could not be probed, or it refused the request.
    Web(WebError),
}

/// Failure of a byte source.
///
/// A deterministic algorithm that reached the end of its capacity and a custom source that failed
/// both report here, without pretending to be a system entropy fault. Only a failure that started at
/// host entropy carries the platform differences, through [`Entropy`](Self::Entropy).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SourceError {
    /// The source reached the end of its capacity.
    Exhausted,
    /// The source exists but is currently unusable.
    Unavailable,
    /// The source failed without reporting a platform code.
    Failed,
    /// The failure came from a host entropy backend.
    Entropy(EntropyError),
}

/// Failure of a sampling operation.
///
/// Parameter and numerical errors use their own variants and never carry a platform code, because a
/// range or a probability is not a platform capability. A failure that started at a byte source is
/// propagated unchanged through [`Source`](Self::Source), so no layer loses the original error.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RandomError {
    /// The range bounds are empty or reversed.
    InvalidRange,
    /// A rational probability has a zero denominator or an out-of-range numerator.
    InvalidProbability,
    /// A distribution parameter model is internally inconsistent or unusable.
    InvalidParameter,
    /// A length or position cannot be represented by the profile.
    LengthOverflow,
    /// A sampled value cannot be expressed by the numerical contract of its distribution.
    NumericalFailure,
    /// The byte source failed.
    Source(SourceError),
}

impl From<EntropyError> for SourceError {
    fn from(error: EntropyError) -> Self {
        Self::Entropy(error)
    }
}

impl From<SourceError> for RandomError {
    fn from(error: SourceError) -> Self {
        Self::Source(error)
    }
}

impl Display for WebError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Probe => f.write_str("the host has no crypto property"),
            Self::Rejected => f.write_str("the host refused the entropy request"),
        }
    }
}

impl Display for EntropyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported => f.write_str("this target has no implemented entropy backend"),
            Self::Unavailable => f.write_str("host entropy is currently unavailable"),
            Self::Failed => f.write_str("the host entropy backend failed"),
            Self::Errno(code) => write!(f, "the host entropy backend reported errno {code}"),
            Self::NtStatus(code) => write!(f, "the host entropy backend reported NTSTATUS {code:#010x}"),
            Self::Wasi(code) => write!(f, "the host entropy backend reported WASI errno {code}"),
            Self::Web(error) => write!(f, "the Web host entropy bridge failed: {error}"),
        }
    }
}

impl Display for SourceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exhausted => f.write_str("the source reached the end of its capacity"),
            Self::Unavailable => f.write_str("the source is currently unavailable"),
            Self::Failed => f.write_str("the source failed"),
            Self::Entropy(error) => write!(f, "the host entropy source failed: {error}"),
        }
    }
}

impl Display for RandomError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRange => f.write_str("the range bounds are empty or reversed"),
            Self::InvalidProbability => f.write_str("the rational probability is invalid"),
            Self::InvalidParameter => f.write_str("the distribution parameters are invalid"),
            Self::LengthOverflow => f.write_str("the length or position is outside the profile"),
            Self::NumericalFailure => f.write_str("the sampled value is outside the numerical contract"),
            Self::Source(error) => write!(f, "the byte source failed: {error}"),
        }
    }
}

/// The `std` feature exposes the errors through `core::error::Error`, which is the same trait as
/// `std::error::Error` in the standard library. The implementation keeps the causal chain of a
/// failure that crossed a layer boundary, so `source()` still points at the original backend error.
#[cfg(feature = "std")]
impl core::error::Error for WebError {}

/// The `std` feature exposes the errors through `core::error::Error`.
#[cfg(feature = "std")]
impl core::error::Error for EntropyError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Web(error) => Some(error),
            Self::Unsupported | Self::Unavailable | Self::Failed | Self::Errno(_) | Self::NtStatus(_) | Self::Wasi(_) => None,
        }
    }
}

/// The `std` feature exposes the errors through `core::error::Error`.
#[cfg(feature = "std")]
impl core::error::Error for SourceError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Entropy(error) => Some(error),
            Self::Exhausted | Self::Unavailable | Self::Failed => None,
        }
    }
}

/// The `std` feature exposes the errors through `core::error::Error`.
#[cfg(feature = "std")]
impl core::error::Error for RandomError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Source(error) => Some(error),
            Self::InvalidRange
            | Self::InvalidProbability
            | Self::InvalidParameter
            | Self::LengthOverflow
            | Self::NumericalFailure => None,
        }
    }
}
