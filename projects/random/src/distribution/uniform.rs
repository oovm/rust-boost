//! Uniform distributions over verified integer intervals.
//!
//! The type is `Uniform<T>`, one value object per concrete integer endpoint. Each value of the
//! interval has the same mass `1/n`. Sampling reuses the shared rejection kernel of the profile
//! unchanged, so a distribution object and the matching `range_u32`/`range_i64` call on
//! [`Random`](crate::Random) produce the same value and consume the same bytes.
//!
//! The constructor takes standard [`RangeBounds`] and normalizes them once, so a sampling call never
//! re-validates its interval. The concrete inherent implementations are generated per endpoint type,
//! because the numeric width of the interval count and the signed order-preserving mapping differ.

use core::ops::RangeBounds;

use crate::{error::RandomError, sampling, source::RandomSource};

/// Uniform distribution over an inclusive integer interval.
///
/// The fields are private: a value is produced by the concrete constructor of its endpoint type, and
/// a successful construction is an immutable, already valid interval. The type itself carries no
/// arithmetic, so the signed order-preserving mapping stays inside the private sampling functions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Uniform<T> {
    start: T,
    end: T,
}

/// Declares the inherent implementation for one concrete integer endpoint.
///
/// `$wide` is the integer width used for the interval count, so a full-domain interval of a 32-bit or
/// 64-bit endpoint is expressed without overflowing the endpoint type itself. `$kernel` is the shared
/// sampling entry that maps the ordered interval through the rejection loop.
macro_rules! uniform_impl {
    ($value:ty, $wide:ty, $kernel:path) => {
        impl Uniform<$value> {
            /// Creates the distribution over a standard integer range.
            ///
            /// The bounds are normalized during construction: an inclusive bound is kept, an excluded
            /// lower bound is checked up by one, an excluded upper bound is checked down by one, and an
            /// unbounded side maps to the extreme of the endpoint type. An excluded extreme, a reversed
            /// interval, or an empty one fails with [`RandomError::InvalidRange`] before any byte is
            /// consumed.
            pub fn new<B: RangeBounds<$value>>(bounds: B) -> Result<Self, RandomError> {
                let start = match bounds.start_bound() {
                    core::ops::Bound::Included(value) => *value,
                    core::ops::Bound::Excluded(value) => value.checked_add(1).ok_or(RandomError::InvalidRange)?,
                    core::ops::Bound::Unbounded => <$value>::MIN,
                };
                let end = match bounds.end_bound() {
                    core::ops::Bound::Included(value) => *value,
                    core::ops::Bound::Excluded(value) => value.checked_sub(1).ok_or(RandomError::InvalidRange)?,
                    core::ops::Bound::Unbounded => <$value>::MAX,
                };
                if start > end {
                    return Err(RandomError::InvalidRange);
                }
                Ok(Self { start, end })
            }

            /// Returns the inclusive interval of the support set.
            pub fn bounds(&self) -> ($value, $value) {
                (self.start, self.end)
            }

            /// Samples one value with the shared unbiased rejection kernel.
            ///
            /// A single-value interval consumes no byte, and a full-domain interval consumes exactly
            /// one word of the endpoint width. A source failure is propagated unchanged.
            pub fn sample<S: RandomSource + ?Sized>(&self, source: &mut S) -> Result<$value, RandomError> {
                $kernel(source, self.start, self.end).map_err(RandomError::Source)
            }

            /// Returns the probability mass of one value as a float approximation.
            ///
            /// The exact law is `1/n` over the interval. A value outside the support set has mass
            /// zero, and one value inside the support set is `1/n`. The count is computed in a wider
            /// integer first, so a full-domain interval does not overflow.
            pub fn mass(&self, value: $value) -> f64 {
                if value < self.start || value > self.end {
                    return 0.0;
                }
                1.0 / self.count() as f64
            }

            /// Returns the number of values of the support set.
            fn count(&self) -> $wide {
                (self.end as $wide - self.start as $wide) + 1
            }
        }
    };
}

uniform_impl!(u32, u128, sampling::sample_inclusive);
uniform_impl!(u64, u128, sampling::sample_inclusive);
uniform_impl!(i32, i128, sampling::sample_i32);
uniform_impl!(i64, i128, sampling::sample_i64);
