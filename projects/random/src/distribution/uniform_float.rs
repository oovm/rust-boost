//! Uniform distributions over float intervals.
//!
//! The model is the numeric interval `[low, high)`, so the density inside the support is
//! `1 / (high - low)`. Sampling maps one uniform value from the profile onto that interval. The
//! mapping uses only IEEE arithmetic, so it is bit-reproducible on every target that implements
//! IEEE 754 binary32 and binary64, unlike the distributions that need a transcendental function.
//!
//! Two numerical properties are deliberate and documented rather than hidden:
//!
//! - The support is never left. A value that would round up to `high` is moved to the largest
//!   representable value strictly below `high`, so the excluded upper bound keeps zero mass instead
//!   of being covered by a clamp across the whole interval.
//! - An interval narrower than a representable step, or one whose width is subnormal, concentrates
//!   its mass on the representable values that remain inside the support. That is the faithful
//!   result for such an interval, not a bias introduced by the mapping.

use crate::{error::RandomError, sampling, source::RandomSource};

/// Declares one uniform distribution over a float endpoint type.
///
/// `$below_zero` is the bit pattern of the largest value strictly below either zero, which the
/// predecessor step needs when it is asked about zero itself.
macro_rules! uniform_float {
    ($name:ident, $ty:ty, $bits:ty, $below_zero:expr, $unit:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct $name {
            low: $ty,
            high: $ty,
        }

        impl $name {
            /// Creates the distribution over the half-open interval `[low, high)`.
            ///
            /// Both bounds must be finite and `low` must be strictly below `high`. A non-finite
            /// bound, an empty interval, or a reversed one fails with
            /// [`RandomError::InvalidParameter`] during
            /// construction, so a sampling call never has to re-validate its parameters.
            pub fn new(low: $ty, high: $ty) -> Result<Self, RandomError> {
                if low.is_finite() && high.is_finite() && low < high {
                    return Ok(Self { low, high });
                }
                Err(RandomError::InvalidParameter)
            }

            /// Returns the inclusive lower bound of the support.
            pub fn low(&self) -> $ty {
                self.low
            }

            /// Returns the excluded upper bound of the support.
            pub fn high(&self) -> $ty {
                self.high
            }

            /// Samples one value from the support.
            ///
            /// The mapping is `low + u * (high - low)` with `u` from `[0, 1)`, except for an interval
            /// whose width is not representable: there the two scaled bounds are combined instead, so
            /// a cross-zero interval such as `[-MAX, MAX]` still produces a finite value. Both forms
            /// keep the result inside `[low, high)`.
            pub fn sample<S: RandomSource + ?Sized>(&self, source: &mut S) -> Result<$ty, RandomError> {
                let unit = sampling::$unit(source).map_err(RandomError::Source)?;
                let width = self.high - self.low;
                let sampled = if width.is_finite() {
                    // `u < 1`, so the exact product is below the width and only rounding can reach it.
                    let scaled = unit * width;
                    let scaled = if scaled >= width { Self::previous(scaled) } else { scaled };
                    self.low + scaled
                }
                else {
                    // The width exceeded the largest finite value, so the direct form would produce
                    // an infinity. Two scaled bounds stay finite and combine into the same interval.
                    self.low * (1.0 - unit) + self.high * unit
                };
                if sampled >= self.high { Ok(Self::previous(sampled)) } else { Ok(sampled) }
            }

            /// Returns the probability density inside the support.
            ///
            /// The value is computed from half of the width, so an interval whose width exceeds the
            /// largest finite value still returns its exact density instead of underflowing to zero.
            /// A query outside the support returns zero, because the density there really is zero.
            pub fn density(&self) -> $ty {
                0.5 / self.half_width()
            }

            /// Returns the cumulative probability below `value` as a float approximation.
            ///
            /// The result is `0` at or below the lower bound and `1` at or above the upper bound.
            /// Like the density, it is computed from half of the width so a very wide interval still
            /// reports the correct fraction.
            pub fn cdf(&self, value: $ty) -> $ty {
                if value < self.low {
                    return 0.0;
                }
                if value >= self.high {
                    return 1.0;
                }
                (value * 0.5 - self.low * 0.5) / self.half_width()
            }

            /// Returns half of the interval width, which stays representable for every interval this
            /// type accepts.
            fn half_width(&self) -> $ty {
                self.high * 0.5 - self.low * 0.5
            }

            /// Returns the largest representable value strictly below `value`.
            ///
            /// The step moves one unit of raw encoding toward negative infinity, so the sign and the
            /// magnitude both follow the ordering of the format. Zero has to be handled separately,
            /// because its encoding is shared with the smallest negative subnormal.
            fn previous(value: $ty) -> $ty {
                if value == 0.0 {
                    return <$ty>::from_bits($below_zero);
                }
                if value > 0.0 { <$ty>::from_bits(value.to_bits() - 1) } else { <$ty>::from_bits(value.to_bits() + 1) }
            }
        }
    };
}

uniform_float!(
    UniformF32,
    f32,
    u32,
    0x8000_0001_u32,
    unit_f32,
    "Uniform distribution over the half-open `f32` interval `[low, high)`."
);
uniform_float!(
    UniformF64,
    f64,
    u64,
    0x8000_0000_0000_0001_u64,
    unit_f64,
    "Uniform distribution over the half-open `f64` interval `[low, high)`."
);
