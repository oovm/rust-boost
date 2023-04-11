//! Bernoulli distribution over a rational probability.

use crate::{
    error::{RandomError, RandomErrorKind},
    range::RangeEndpoint,
    source::RandomSource,
};

/// Distribution of one boolean event with the rational probability `numerator / denominator`.
///
/// The parameters are validated once, during construction. The fraction is kept as given instead
/// of being reduced, because the denominator is part of the sampling mapping: an equivalent
/// probability written with another denominator expresses the same law but not the same
/// value-for-value coupling with the byte sequence. A degenerate probability of zero or one is a
/// valid model, and it consumes no byte while sampling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bernoulli {
    numerator: u64,
    denominator: u64,
}

impl Bernoulli {
    /// Creates the distribution from the rational probability `numerator / denominator`.
    ///
    /// A zero denominator or a numerator above the denominator fails with
    /// [`ErrorKind::InvalidProbability`](crate::RandomErrorKind::InvalidProbability).
    pub fn new(numerator: u64, denominator: u64) -> Result<Self, RandomError> {
        if denominator == 0 || numerator > denominator {
            return Err(RandomError::new(RandomErrorKind::InvalidProbability));
        }
        Ok(Self { numerator, denominator })
    }

    /// Returns the numerator of the probability.
    pub fn numerator(&self) -> u64 {
        self.numerator
    }

    /// Returns the denominator of the probability.
    pub fn denominator(&self) -> u64 {
        self.denominator
    }

    /// Samples the event with the shared unbiased integer kernel.
    ///
    /// The mapping is the one used by [`Stream::chance`](crate::Stream::chance): a probability of
    /// zero returns `false` and a probability of one returns `true` without consuming any byte.
    pub fn sample<S: RandomSource + ?Sized>(&self, source: &mut S) -> Result<bool, RandomError> {
        if self.numerator == 0 {
            return Ok(false);
        }
        if self.numerator == self.denominator {
            return Ok(true);
        }
        Ok(u64::sample_bounded(source, 0, self.denominator - 1)? < self.numerator)
    }

    /// Returns the probability mass of one outcome as a float approximation.
    ///
    /// The mass of `false` is formed from `denominator - numerator` in the integer domain first, so
    /// an extreme probability is not rounded twice.
    pub fn mass(&self, outcome: bool) -> f64 {
        let favourable = if outcome { self.numerator } else { self.denominator - self.numerator };
        favourable as f64 / self.denominator as f64
    }
}
