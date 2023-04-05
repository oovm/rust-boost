//! Categorical distribution over borrowed integer weights.

use crate::{
    error::{Error, ErrorKind},
    range::RangeEndpoint,
    source::RandomSource,
};

/// Distribution over categories given by integer weights, sampled as a category index.
///
/// The weights are borrowed for the lifetime of the object, so the default `no_std` core needs no
/// allocation. The sum is validated once, during construction, and the sample is obtained by
/// scanning the cumulative weights in their original order: the first category whose cumulative
/// weight strictly exceeds the ticket is returned. A zero weight is never selected, and the
/// category numbering and its order are not changed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Categorical<'a> {
    weights: &'a [u64],
    total: u64,
}

impl<'a> Categorical<'a> {
    /// Creates the distribution from a non-empty weight slice with a positive, representable sum.
    ///
    /// An empty slice, an all-zero slice, or a sum that does not fit in `u64` fails with
    /// [`ErrorKind::InvalidParameter`](crate::ErrorKind::InvalidParameter).
    pub fn new(weights: &'a [u64]) -> Result<Self, Error> {
        if weights.is_empty() {
            return Err(Error::new(ErrorKind::InvalidParameter));
        }
        let mut total = 0u64;
        for weight in weights {
            total = total.checked_add(*weight).ok_or(Error::new(ErrorKind::InvalidParameter))?;
        }
        if total == 0 {
            return Err(Error::new(ErrorKind::InvalidParameter));
        }
        Ok(Self { weights, total })
    }

    /// Returns the borrowed weights.
    pub fn weights(&self) -> &'a [u64] {
        self.weights
    }

    /// Returns the validated sum of the weights.
    pub fn total(&self) -> u64 {
        self.total
    }

    /// Samples one category index with the shared unbiased integer kernel.
    ///
    /// One ticket is drawn uniformly from `[0, total)` and matched against the cumulative weights.
    /// A source failure is reported without inventing a category. If the recorded sum ever
    /// disagreed with the scanned weights, the call fails instead of returning a value that the
    /// weights do not justify.
    pub fn sample<S: RandomSource + ?Sized>(&self, source: &mut S) -> Result<u64, Error> {
        let ticket = u64::sample_bounded(source, 0, self.total - 1)?;
        let mut cumulative = 0u64;
        for (index, weight) in self.weights.iter().enumerate() {
            cumulative += *weight;
            if cumulative > ticket {
                return Ok(index as u64);
            }
        }
        Err(Error::new(ErrorKind::InvalidParameter))
    }

    /// Returns the probability mass of one category index as a float approximation.
    ///
    /// An index outside the weight slice has mass zero.
    pub fn mass(&self, category: u64) -> f64 {
        match usize::try_from(category).ok().and_then(|index| self.weights.get(index)) {
            Some(weight) => *weight as f64 / self.total as f64,
            None => 0.0,
        }
    }
}
