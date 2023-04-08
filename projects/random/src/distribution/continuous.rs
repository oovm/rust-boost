//! Normal and exponential distributions.
//!
//! Both models need a transcendental function, so both sample with the crate's own arithmetic kernel
//! instead of a platform math library. The kernel is IEEE arithmetic, but an error bound rather than
//! a bit-for-bit promise describes its output, so both distributions are published as a numerical
//! contract: the domain, the support, and the statistical quality are fixed, while the exact bits
//! depend on the rounding of the target.
//!
//! The continuous density is the model, so these types expose density and CDF where the format can
//! express them. A capability that cannot be expressed is absent rather than faked: the normal CDF
//! needs an error function, which this crate does not implement, so `Normal` has no `cdf`.

use crate::{
    error::{Error, ErrorKind},
    math, sampling,
    source::RandomSource,
};

/// Square root of two pi, used by the normal density.
const SQRT_TWO_PI: f64 = 2.506_628_274_631_000_7;

/// Distribution of a normal variable with a mean and a standard deviation.
///
/// The parameters are the mean and the **standard deviation**, not the variance. The sampling
/// strategy is Marsaglia's polar method, which draws two symmetric grid values, rejects the pair
/// outside the unit circle, and transforms the accepted one. Nothing is cached between calls, so the
/// parameter object holds no sampling state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Normal {
    mean: f64,
    std_dev: f64,
}

impl Normal {
    /// Creates the distribution from a finite `mean` and a finite, strictly positive `std_dev`.
    ///
    /// A non-finite parameter or a non-positive deviation fails with
    /// [`ErrorKind::InvalidParameter`](crate::ErrorKind::InvalidParameter) during construction.
    pub fn new(mean: f64, std_dev: f64) -> Result<Self, Error> {
        if mean.is_finite() && std_dev.is_finite() && std_dev > 0.0 {
            return Ok(Self { mean, std_dev });
        }
        Err(Error::new(ErrorKind::InvalidParameter))
    }

    /// Returns the mean of the distribution.
    pub fn mean(&self) -> f64 {
        self.mean
    }

    /// Returns the standard deviation of the distribution.
    pub fn std_dev(&self) -> f64 {
        self.std_dev
    }

    /// Returns the variance of the distribution.
    ///
    /// A deviation large enough for its square to leave the format returns infinity, which reports
    /// that the value is not representable rather than inventing a finite one.
    pub fn variance(&self) -> f64 {
        self.std_dev * self.std_dev
    }

    /// Samples one value from the support.
    ///
    /// The rejection loop repeats while the pair is at the origin or outside the unit circle. A
    /// source failure is reported as it happens, and a drawn value whose position and scale cannot be
    /// represented fails with [`ErrorKind::NumericalFailure`](crate::ErrorKind::NumericalFailure)
    /// instead of returning a non-finite sample.
    pub fn sample<S: RandomSource + ?Sized>(&self, source: &mut S) -> Result<f64, Error> {
        loop {
            let first = symmetric(source)?;
            let second = symmetric(source)?;
            let squared = first * first + second * second;
            if squared == 0.0 || squared >= 1.0 {
                continue;
            }
            let standard = first * math::sqrt(-2.0 * math::ln(squared) / squared);
            let value = self.mean + self.std_dev * standard;
            if !value.is_finite() {
                return Err(Error::new(ErrorKind::NumericalFailure));
            }
            return Ok(value);
        }
    }

    /// Returns the probability density at `value`.
    pub fn density(&self, value: f64) -> f64 {
        let deviation = (value - self.mean) / self.std_dev;
        math::exp(-0.5 * deviation * deviation) / (self.std_dev * SQRT_TWO_PI)
    }
}

/// Distribution of the waiting time of a Poisson process with a given rate.
///
/// The sampling strategy is the inverse transform `-ln(u) / rate` with `u` drawn from the grid
/// `(0, 1]`, so the logarithm never receives zero and no sample can be infinite by construction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Exponential {
    rate: f64,
}

impl Exponential {
    /// Creates the distribution from a finite, strictly positive `rate`.
    pub fn new(rate: f64) -> Result<Self, Error> {
        if rate.is_finite() && rate > 0.0 {
            return Ok(Self { rate });
        }
        Err(Error::new(ErrorKind::InvalidParameter))
    }

    /// Returns the rate of the distribution.
    pub fn rate(&self) -> f64 {
        self.rate
    }

    /// Returns the mean of the distribution, which is `1 / rate`.
    pub fn mean(&self) -> f64 {
        1.0 / self.rate
    }

    /// Returns the variance of the distribution, which is `1 / rate^2`.
    pub fn variance(&self) -> f64 {
        let mean = self.mean();
        mean * mean
    }

    /// Samples one value from the support.
    ///
    /// A source failure is reported as it happens, and a value whose quotient cannot be represented
    /// fails with [`ErrorKind::NumericalFailure`](crate::ErrorKind::NumericalFailure) instead of
    /// returning a non-finite sample.
    pub fn sample<S: RandomSource + ?Sized>(&self, source: &mut S) -> Result<f64, Error> {
        let word = sampling::read_u64(source)?;
        // The top 53 bits are stepped to the largest value strictly below one and scaled by `2^-53`,
        // which keeps `u` inside `(0, 1]` with the resolution of the format.
        let unit = (((word >> 11) + 1) as f64) * (1.0 / 9_007_199_254_740_992.0);
        let value = -math::ln(unit) / self.rate;
        if !value.is_finite() {
            return Err(Error::new(ErrorKind::NumericalFailure));
        }
        Ok(value)
    }

    /// Returns the probability density at `value`.
    ///
    /// The density of a negative `value` is zero, because the support starts at zero.
    pub fn density(&self, value: f64) -> f64 {
        if value < 0.0 {
            return 0.0;
        }
        self.rate * math::exp(-self.rate * value)
    }

    /// Returns the cumulative probability below `value`.
    ///
    /// A negative `value` has cumulative probability zero, because the support starts at zero. The
    /// result is accurate to a few units in the last place of one: the subtraction from one cancels
    /// when the probability is small, so a relative bound near zero would be wrong to promise.
    pub fn cdf(&self, value: f64) -> f64 {
        if value < 0.0 {
            return 0.0;
        }
        1.0 - math::exp(-self.rate * value)
    }
}

/// Draws one value from the symmetric grid strictly inside `(-1, 1)`.
///
/// The grid holds every odd multiple of `2^-52` in that range, so it is symmetric about zero and
/// every value has the same probability.
fn symmetric<S: RandomSource + ?Sized>(source: &mut S) -> Result<f64, Error> {
    let word = sampling::read_u64(source)?;
    let step = (word >> 12) as f64;
    Ok((2.0 * step + 1.0) * (1.0 / 4_503_599_627_370_496.0) - 1.0)
}
