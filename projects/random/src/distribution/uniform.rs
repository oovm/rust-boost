//! Uniform distributions over verified integer intervals.
//!
//! Each value of the interval has the same mass `1/n`. Sampling reuses the shared rejection kernel
//! of the profile unchanged, so a distribution object and the matching `range` call on a stream
//! produce the same value and consume the same bytes.

use crate::{
    error::{Error, ErrorKind},
    range::{RangeEndpoint, SampleRange},
    source::RandomSource,
};

/// Declares one uniform distribution over an integer endpoint type.
///
/// `$wide` is the integer width used for the interval count, so a full-domain interval of a 32-bit
/// or 64-bit endpoint is expressed without overflowing the endpoint type itself.
macro_rules! uniform_distribution {
    ($name:ident, $value:ty, $wide:ty, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct $name {
            start: $value,
            end: $value,
        }

        impl $name {
            /// Creates the distribution over an already validated inclusive interval.
            ///
            /// The bounds come from the same range types accepted by
            /// [`Stream::range`](crate::Stream::range). An empty or reversed interval fails with
            /// [`ErrorKind::InvalidRange`](crate::ErrorKind::InvalidRange) during construction, so
            /// a sampling call never has to re-validate its parameters.
            pub fn new<B: SampleRange<$value>>(bounds: B) -> Result<Self, Error> {
                let (start, end) = bounds.bounds()?;
                if end < start {
                    return Err(Error::new(ErrorKind::InvalidRange));
                }
                Ok(Self { start, end })
            }

            /// Returns the inclusive interval of the support set.
            pub fn bounds(&self) -> ($value, $value) {
                (self.start, self.end)
            }

            /// Samples one value with the shared unbiased rejection kernel.
            ///
            /// A single-value interval consumes no byte, and a full-domain interval consumes
            /// exactly one word of the endpoint width.
            pub fn sample<S: RandomSource + ?Sized>(&self, source: &mut S) -> Result<$value, Error> {
                <$value>::sample_bounded(source, self.start, self.end)
            }

            /// Returns the probability mass of one value as a float approximation.
            ///
            /// The exact law is `1/n` over the interval. A value outside the support set has mass
            /// zero, and one value inside the support set is `1/n`. The count is computed in a
            /// wider integer first, so a full-domain interval does not overflow.
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

uniform_distribution!(UniformU32, u32, u128, "Uniform distribution over an inclusive `u32` interval.");
uniform_distribution!(UniformU64, u64, u128, "Uniform distribution over an inclusive `u64` interval.");
uniform_distribution!(UniformI32, i32, i128, "Uniform distribution over an inclusive `i32` interval.");
uniform_distribution!(UniformI64, i64, i128, "Uniform distribution over an inclusive `i64` interval.");
