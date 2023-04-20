//! Contract and accuracy tests for the normal and exponential distributions.
//!
//! The arithmetic kernel is private, so its error is measured through the public sampling entries
//! instead of an internal hook: a caller-programmed word source supplies the exact words, the test
//! recomputes the expected transform with the host standard library, and the difference is bounded in
//! units in the last place of the result. That is the evidence behind the accuracy claim, and it does
//! not require making the kernel public.
//!
//! The statistical cases use a fixed seed, so they are deterministic: a failing threshold means the
//! distribution changed, not that one run happened to differ.

use random::{
    ChaCha12, Random, RandomError, RandomSource, SourceError,
    distribution::{Exponential, Normal},
};

/// Source that plays a programmed sequence of whole words.
struct WordQueue {
    bytes: Vec<u8>,
    position: usize,
}

impl WordQueue {
    /// Programs a sequence of little-endian `u64` words.
    fn of_u64(words: &[u64]) -> Self {
        let bytes = words.iter().flat_map(|word| word.to_le_bytes()).collect();
        Self { bytes, position: 0 }
    }

    /// Returns how many bytes were consumed.
    fn consumed(&self) -> usize {
        self.position
    }
}

impl RandomSource for WordQueue {
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), SourceError> {
        let end = self.position + dst.len();
        if end > self.bytes.len() {
            return Err(SourceError::Exhausted);
        }
        dst.copy_from_slice(&self.bytes[self.position..end]);
        self.position = end;
        Ok(())
    }
}

/// Creates a source that only provides a stable identity for statistical cases.
fn fixture() -> Random<ChaCha12> {
    Random::new(ChaCha12::from_u64(0x5eed_2a01, 11))
}

/// Turns a word into the `(0, 1]` grid value used by the exponential distribution.
fn exponential_unit(word: u64) -> f64 {
    (((word >> 11) + 1) as f64) * (1.0 / 9_007_199_254_740_992.0)
}

/// Turns a word into the symmetric grid value used by the normal distribution.
fn normal_grid(word: u64) -> f64 {
    (2.0 * ((word >> 12) as f64) + 1.0) * (1.0 / 4_503_599_627_370_496.0) - 1.0
}

/// Returns the distance between two values in units in the last place of `reference`.
fn ulp_distance(value: f64, reference: f64) -> f64 {
    if value == reference {
        return 0.0;
    }
    (value - reference).abs() / (reference.abs().max(f64::MIN_POSITIVE) * f64::EPSILON)
}

#[test]
fn invalid_parameters_are_rejected() {
    assert_eq!(Normal::new(f64::NAN, 1.0).unwrap_err(), RandomError::InvalidParameter);
    assert_eq!(Normal::new(0.0, f64::INFINITY).unwrap_err(), RandomError::InvalidParameter);
    assert_eq!(Normal::new(0.0, 0.0).unwrap_err(), RandomError::InvalidParameter);
    assert_eq!(Normal::new(0.0, -1.0).unwrap_err(), RandomError::InvalidParameter);

    assert_eq!(Exponential::new(f64::NAN).unwrap_err(), RandomError::InvalidParameter);
    assert_eq!(Exponential::new(f64::INFINITY).unwrap_err(), RandomError::InvalidParameter);
    assert_eq!(Exponential::new(0.0).unwrap_err(), RandomError::InvalidParameter);
    assert_eq!(Exponential::new(-0.5).unwrap_err(), RandomError::InvalidParameter);

    let normal = Normal::new(3.0, 2.0).unwrap();
    assert_eq!(normal.mean(), 3.0);
    assert_eq!(normal.std_dev(), 2.0);
    assert_eq!(normal.variance(), 4.0);

    let exponential = Exponential::new(4.0).unwrap();
    assert_eq!(exponential.rate(), 4.0);
    assert_eq!(exponential.mean(), 0.25);
    assert_eq!(exponential.variance(), 0.0625);
}

#[test]
fn exponential_sampling_matches_the_inverse_transform() {
    // The words cover the smallest possible output, the largest one, and interior values.
    let words = [u64::MAX, 0, 1, 1 << 11, 12345 << 11, 0x1234_5678_9abc_def0, (1 << 40) << 11];
    let rate = 2.5;
    let distribution = Exponential::new(rate).unwrap();

    for word in words {
        let mut source = WordQueue::of_u64(&[word, word]);
        let sampled = distribution.sample(&mut source).unwrap();
        let expected = -exponential_unit(word).ln() / rate;
        assert!(sampled >= 0.0, "the support starts at zero, got {sampled}");
        assert!(ulp_distance(sampled, expected) <= 4.0, "word {word:#x} produced {sampled} instead of {expected}");
        assert_eq!(source.consumed(), 8, "one exponential sample must consume exactly one word");
    }
}

#[test]
fn normal_sampling_matches_the_polar_transform() {
    // Each pair is inside the unit circle, so the transform is applied to exactly two words. The grid
    // steps are `s << 12`, because the distribution reads the top 52 bits as the step.
    let step = |s: u64| s << 12;
    let pairs = [(step(3 << 50), step(1 << 50)), (step(1 << 51), step(1 << 51)), (step(1 << 50), step(3 << 50))];
    let mean = -1.5;
    let std_dev = 0.75;
    let distribution = Normal::new(mean, std_dev).unwrap();

    for (first, second) in pairs {
        let mut source = WordQueue::of_u64(&[first, second, first, second]);
        let sampled = distribution.sample(&mut source).unwrap();

        let u = normal_grid(first);
        let v = normal_grid(second);
        let squared = u * u + v * v;
        assert!(squared > 0.0 && squared < 1.0, "the fixture pair is not inside the unit circle");
        let expected = mean + std_dev * u * (-2.0 * squared.ln() / squared).sqrt();
        assert!(ulp_distance(sampled, expected) <= 4.0, "the pair produced {sampled} instead of {expected}");
        assert_eq!(source.consumed(), 16, "an accepted pair must consume exactly two words");
    }
}

#[test]
fn normal_rejection_consumes_the_rejected_pair() {
    // Both grids near their extremes give a squared radius above one, so the pair is rejected and the
    // next pair decides the sample.
    let rejected = (u64::MAX, u64::MAX);
    let accepted = 3u64 << 62;
    let mut source = WordQueue::of_u64(&[rejected.0, rejected.1, accepted, accepted]);
    let distribution = Normal::new(0.0, 1.0).unwrap();
    let sampled = distribution.sample(&mut source).unwrap();
    assert!(sampled.is_finite());
    assert_eq!(source.consumed(), 32, "the rejected pair was not consumed");

    // The grid holds only odd multiples of `2^-52` shifted away from zero, so a pair at the origin
    // cannot occur and the zero-radius guard never has to run. The pair closest to zero is accepted.
    let closest = (1u64 << 51) << 12;
    let mut source = WordQueue::of_u64(&[closest, closest, closest, closest]);
    assert!(distribution.sample(&mut source).unwrap().is_finite());
    assert_eq!(source.consumed(), 16);
}

#[test]
fn the_exponential_queries_match_the_closed_form() {
    let distribution = Exponential::new(2.0).unwrap();

    // `1 - exp(-2 * ln(2) / 2)` is exactly one half, so the CDF is checked at an exact point.
    assert_eq!(distribution.cdf(0.0), 0.0);
    assert!((distribution.cdf(0.5 * 2.0f64.ln()) - 0.5).abs() < 1e-15);
    assert_eq!(distribution.density(-1.0), 0.0);
    assert!((distribution.density(0.0) - 2.0).abs() < f64::EPSILON);

    for value in [0.05f64, 0.25, 1.0, 3.0, 12.0, 40.0] {
        let expected_density = 2.0 * (-2.0 * value).exp();
        let expected_cdf = 1.0 - (-2.0 * value).exp();
        assert!(ulp_distance(distribution.density(value), expected_density) <= 4.0);
        // The CDF is a subtraction from one, so a small probability cancels and the honest bound is
        // absolute, measured in units of the last place of one.
        assert!((distribution.cdf(value) - expected_cdf).abs() <= 4.0 * f64::EPSILON, "the CDF at {value} is outside the absolute bound");
    }
}

#[test]
fn the_normal_density_matches_the_closed_form() {
    let distribution = Normal::new(1.0, 2.0).unwrap();
    let peak = 1.0 / (2.0 * core::f64::consts::PI).sqrt() / 2.0;
    assert!(ulp_distance(distribution.density(1.0), peak) <= 4.0);

    // One standard deviation away the density is the peak times `exp(-1/2)`.
    for offset in [2.0f64, -2.0, 4.0, -4.0] {
        let expected = peak * (-0.5 * (offset / 2.0).powi(2)).exp();
        assert!(ulp_distance(distribution.density(1.0 + offset), expected) <= 8.0, "the density at offset {offset} is outside the bound");
    }
}

#[test]
fn the_exponential_statistics_hold_for_a_fixed_seed() {
    let distribution = Exponential::new(2.0).unwrap();
    let mut random = fixture();
    let draws = 20_000;
    let mut sum = 0.0;
    for _ in 0..draws {
        let value = distribution.sample(&mut random).unwrap();
        assert!(value >= 0.0 && value.is_finite(), "the support starts at zero, got {value}");
        sum += value;
    }
    let mean = sum / f64::from(draws);
    assert!((mean - 0.5).abs() < 0.02, "the sample mean {mean} left the bound around 0.5");
}

#[test]
fn the_normal_statistics_and_symmetry_hold_for_a_fixed_seed() {
    let distribution = Normal::new(3.0, 2.0).unwrap();
    let mut random = fixture();
    let draws = 20_000;
    let mut sum = 0.0;
    let mut squared_sum = 0.0;
    let mut within_one_deviation = 0usize;
    for _ in 0..draws {
        let value = distribution.sample(&mut random).unwrap();
        assert!(value.is_finite());
        sum += value;
        squared_sum += (value - 3.0) * (value - 3.0);
        if (value - 3.0).abs() <= 2.0 {
            within_one_deviation += 1;
        }
    }

    let mean = sum / f64::from(draws);
    let variance = squared_sum / f64::from(draws);
    assert!((mean - 3.0).abs() < 0.05, "the sample mean {mean} left the bound around 3");
    assert!((variance - 4.0).abs() < 0.25, "the sample variance {variance} left the bound around 4");
    let fraction = within_one_deviation as f64 / f64::from(draws);
    assert!((fraction - 0.6827).abs() < 0.02, "the one-sigma fraction {fraction} left the bound");
}

#[test]
fn unrepresentable_samples_are_reported() {
    // A mean at the format maximum cannot survive a positive standard score, so the position-scale
    // step overflows and must be reported instead of returning an infinity.
    let distribution = Normal::new(f64::MAX, f64::MAX).unwrap();
    let mut random = fixture();
    let mut reported = false;
    for _ in 0..256 {
        match distribution.sample(&mut random) {
            Ok(value) => assert!(value.is_finite(), "an overflowed sample was returned as {value}"),
            Err(error) => {
                assert_eq!(error, RandomError::NumericalFailure);
                reported = true;
                break;
            }
        }
    }
    assert!(reported, "the position-scale overflow was never reported");

    // A rate at the smallest subnormal makes the quotient leave the format.
    let distribution = Exponential::new(f64::from_bits(1)).unwrap();
    let mut random = fixture();
    let mut reported = false;
    for _ in 0..256 {
        match distribution.sample(&mut random) {
            Ok(value) => assert!(value.is_finite(), "an overflowed sample was returned as {value}"),
            Err(error) => {
                assert_eq!(error, RandomError::NumericalFailure);
                reported = true;
                break;
            }
        }
    }
    assert!(reported, "the quotient overflow was never reported");
}

#[test]
fn sampling_is_reproducible_within_one_target() {
    // Bit-for-bit equality across targets is not claimed for these two distributions, because their
    // output depends on rounding inside the kernel. Equality within one target still holds.
    let distribution = Normal::new(-0.5, 1.25).unwrap();
    let exponential = Exponential::new(0.75).unwrap();

    let mut left = Random::new(ChaCha12::from_u64(9, 4));
    let mut right = Random::new(ChaCha12::from_u64(9, 4));
    for _ in 0..256 {
        assert_eq!(distribution.sample(&mut left).unwrap().to_bits(), distribution.sample(&mut right).unwrap().to_bits());
        assert_eq!(exponential.sample(&mut left).unwrap().to_bits(), exponential.sample(&mut right).unwrap().to_bits());
    }
}
