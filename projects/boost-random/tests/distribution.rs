//! Contract tests for the first batch of discrete distributions.
//!
//! The tests check three independent properties: the parameter model accepts and rejects the right
//! inputs, the sampling mapping agrees value-for-value and byte-for-byte with the matching
//! convenience entry of the profile, and the reported mass matches the exact integer law it
//! approximates.

use boost_random::{
    ChaCha12, Random, RandomError, RandomSource, SourceError,
    distribution::{Bernoulli, Categorical, Uniform},
};

/// Deterministic source that plays a fixed byte sequence and reports exhaustion at the end.
struct ByteQueue {
    bytes: Vec<u8>,
    position: usize,
}

impl ByteQueue {
    /// Builds a queue from little-endian `u64` words.
    fn of_u64(words: &[u64]) -> Self {
        let bytes = words.iter().flat_map(|word| word.to_le_bytes()).collect();
        Self { bytes, position: 0 }
    }
}

impl RandomSource for ByteQueue {
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

/// Source that always fails.
struct FailingSource;

impl RandomSource for FailingSource {
    fn try_fill_bytes(&mut self, _dst: &mut [u8]) -> Result<(), SourceError> {
        Err(SourceError::Failed)
    }
}

/// Source used by tests that only need a stable identity.
fn fresh() -> Random<ChaCha12> {
    Random::new(ChaCha12::from_u64(0x5eed, 3))
}

/// Reads one word to detect whether previous calls consumed bytes.
fn probe(random: &mut Random<ChaCha12>) -> u32 {
    random.u32().unwrap()
}

#[test]
// The reversed bounds are the subject of this test, not a mistake in an iteration.
#[allow(clippy::reversed_empty_ranges)]
fn uniform_rejects_invalid_bounds() {
    assert_eq!(Uniform::<u32>::new(9u32..3).unwrap_err(), RandomError::InvalidRange);
    assert_eq!(Uniform::<u32>::new(3u32..=2).unwrap_err(), RandomError::InvalidRange);
    assert_eq!(Uniform::<u64>::new(0u64..0).unwrap_err(), RandomError::InvalidRange);
    assert_eq!(Uniform::<i32>::new(4i32..=-4).unwrap_err(), RandomError::InvalidRange);
    assert_eq!(Uniform::<i64>::new(6i64..=5).unwrap_err(), RandomError::InvalidRange);
}

#[test]
fn uniform_samples_and_masses_follow_the_interval() {
    let dice = Uniform::<u32>::new(1u32..=6).unwrap();
    assert_eq!(dice.bounds(), (1, 6));
    assert_eq!(dice.mass(1), 1.0 / 6.0);
    assert_eq!(dice.mass(6), 1.0 / 6.0);
    assert_eq!(dice.mass(0), 0.0);
    assert_eq!(dice.mass(7), 0.0);

    let mut random = fresh();
    for _ in 0..256 {
        let face = dice.sample(&mut random).unwrap();
        assert!((1..=6).contains(&face), "uniform sample left the interval: {face}");
    }

    let signed = Uniform::<i32>::new(-2i32..=0).unwrap();
    assert_eq!(signed.mass(-3), 0.0);
    assert_eq!(signed.mass(0), 1.0 / 3.0);
    let mut random = Random::new(ChaCha12::from_u64(7, 1));
    for _ in 0..256 {
        let value = signed.sample(&mut random).unwrap();
        assert!((-2..=0).contains(&value), "signed uniform sample left the interval: {value}");
    }
}

#[test]
fn uniform_matches_the_range_entry_of_the_profile() {
    let mut through_range = Random::new(ChaCha12::from_u64(42, 0));
    let mut through_uniform = Random::new(ChaCha12::from_u64(42, 0));

    let unsigned = Uniform::<u32>::new(1u32..=6).unwrap();
    let signed = Uniform::<i32>::new(-4i32..=4).unwrap();
    let wide = Uniform::<u64>::new(10u64..40).unwrap();
    let narrow = Uniform::<i64>::new(i64::MIN..=-1).unwrap();

    for _ in 0..64 {
        assert_eq!(unsigned.sample(&mut through_uniform).unwrap(), through_range.range_u32(1..=6).unwrap());
        assert_eq!(signed.sample(&mut through_uniform).unwrap(), through_range.range_i32(-4..=4).unwrap());
        assert_eq!(wide.sample(&mut through_uniform).unwrap(), through_range.range_u64(10..40).unwrap());
        assert_eq!(narrow.sample(&mut through_uniform).unwrap(), through_range.range_i64(i64::MIN..=-1).unwrap());
    }
    assert_eq!(through_uniform.u32().unwrap(), through_range.u32().unwrap());
}

#[test]
fn uniform_consumption_matches_the_profile() {
    let single = Uniform::<u32>::new(7u32..=7).unwrap();
    let mut random = fresh();
    assert_eq!(single.sample(&mut random).unwrap(), 7);
    assert_eq!(probe(&mut random), probe(&mut fresh()));

    let full = Uniform::<u32>::new(0u32..=u32::MAX).unwrap();
    let mut random = fresh();
    full.sample(&mut random).unwrap();
    let mut reference = fresh();
    reference.u32().unwrap();
    assert_eq!(random.u32().unwrap(), reference.u32().unwrap());

    let full = Uniform::<u64>::new(0u64..=u64::MAX).unwrap();
    let mut random = fresh();
    full.sample(&mut random).unwrap();
    let mut reference = fresh();
    reference.u64().unwrap();
    assert_eq!(random.u64().unwrap(), reference.u64().unwrap());

    let full = Uniform::<i32>::new(i32::MIN..=i32::MAX).unwrap();
    let mut random = fresh();
    full.sample(&mut random).unwrap();
    let mut reference = fresh();
    reference.u32().unwrap();
    assert_eq!(random.u32().unwrap(), reference.u32().unwrap());

    let full = Uniform::<i64>::new(i64::MIN..=i64::MAX).unwrap();
    let mut random = fresh();
    full.sample(&mut random).unwrap();
    let mut reference = fresh();
    reference.u64().unwrap();
    assert_eq!(random.u64().unwrap(), reference.u64().unwrap());
}

#[test]
fn uniform_full_domain_mass_is_exact() {
    let u32_full = Uniform::<u32>::new(0u32..=u32::MAX).unwrap();
    assert_eq!(u32_full.mass(0), 1.0 / 4_294_967_296.0);

    let u64_full = Uniform::<u64>::new(0u64..=u64::MAX).unwrap();
    assert_eq!(u64_full.mass(u64::MAX), 1.0 / 18_446_744_073_709_551_616.0);

    let i32_full = Uniform::<i32>::new(i32::MIN..=i32::MAX).unwrap();
    assert_eq!(i32_full.mass(i32::MIN), 1.0 / 4_294_967_296.0);

    let i64_full = Uniform::<i64>::new(i64::MIN..=i64::MAX).unwrap();
    assert_eq!(i64_full.mass(0), 1.0 / 18_446_744_073_709_551_616.0);
}

#[test]
fn uniform_reports_source_failures() {
    let dice = Uniform::<u32>::new(1u32..=6).unwrap();
    assert_eq!(dice.sample(&mut FailingSource).unwrap_err(), RandomError::Source(SourceError::Failed));

    let single = Uniform::<u32>::new(3u32..=3).unwrap();
    assert_eq!(single.sample(&mut FailingSource).unwrap(), 3);
}

#[test]
fn bernoulli_rejects_invalid_probabilities() {
    assert_eq!(Bernoulli::new(1, 0).unwrap_err(), RandomError::InvalidProbability);
    assert_eq!(Bernoulli::new(6, 5).unwrap_err(), RandomError::InvalidProbability);
}

#[test]
fn bernoulli_masses_are_formed_in_the_integer_domain() {
    let event = Bernoulli::new(1, 3).unwrap();
    assert_eq!(event.numerator(), 1);
    assert_eq!(event.denominator(), 3);
    assert_eq!(event.mass(true), 1.0 / 3.0);
    assert_eq!(event.mass(false), 2.0 / 3.0);

    let near_certain = Bernoulli::new(999_999_999_999_999_999, 1_000_000_000_000_000_000).unwrap();
    assert_eq!(near_certain.mass(false), 1.0 / 1_000_000_000_000_000_000.0);
    assert_eq!(near_certain.mass(true) + near_certain.mass(false), 1.0);
}

#[test]
fn bernoulli_matches_the_chance_entry_of_the_profile() {
    let event = Bernoulli::new(1, 3).unwrap();
    let mut through_chance = Random::new(ChaCha12::from_u64(11, 2));
    let mut through_event = Random::new(ChaCha12::from_u64(11, 2));
    for _ in 0..256 {
        assert_eq!(event.sample(&mut through_event).unwrap(), through_chance.chance(1, 3).unwrap());
    }
    assert_eq!(through_event.u32().unwrap(), through_chance.u32().unwrap());
}

#[test]
fn degenerate_bernoulli_consumes_nothing() {
    let never = Bernoulli::new(0, 5).unwrap();
    let always = Bernoulli::new(5, 5).unwrap();
    let mut random = fresh();
    assert!(!never.sample(&mut random).unwrap());
    assert!(always.sample(&mut random).unwrap());
    assert_eq!(probe(&mut random), probe(&mut fresh()));
    assert_eq!(never.mass(true), 0.0);
    assert_eq!(always.mass(false), 0.0);
}

#[test]
fn bernoulli_produces_both_outcomes() {
    let event = Bernoulli::new(1, 3).unwrap();
    let mut random = fresh();
    let mut outcomes = [0usize; 2];
    for _ in 0..256 {
        outcomes[usize::from(event.sample(&mut random).unwrap())] += 1;
    }
    assert!(outcomes[0] > 0 && outcomes[1] > 0, "bernoulli(1, 3) never varied: {outcomes:?}");
}

/// The documented reference mapping of the categorical ticket scan.
fn model_category(weights: &[u64], ticket: u64) -> u64 {
    let mut cumulative = 0u64;
    for (index, weight) in weights.iter().enumerate() {
        cumulative += weight;
        if cumulative > ticket {
            return index as u64;
        }
    }
    panic!("the model requires a ticket below the weight sum");
}

#[test]
fn categorical_masses_follow_the_weight_ratios() {
    let weights = [0u64, 2, 5];
    let category = Categorical::new(&weights).unwrap();
    assert_eq!(category.weights(), &weights);
    assert_eq!(category.total(), 7);
    assert_eq!(category.mass(0), 0.0);
    assert_eq!(category.mass(1), 2.0 / 7.0);
    assert_eq!(category.mass(2), 5.0 / 7.0);
    assert_eq!(category.mass(3), 0.0);
    assert_eq!(category.mass(u64::MAX), 0.0);
}

#[test]
fn categorical_rejects_unusable_weights() {
    assert_eq!(Categorical::new(&[]).unwrap_err(), RandomError::InvalidParameter);
    assert_eq!(Categorical::new(&[0u64, 0]).unwrap_err(), RandomError::InvalidParameter);
    assert_eq!(Categorical::new(&[u64::MAX, 1]).unwrap_err(), RandomError::InvalidParameter);
    assert_eq!(Categorical::new(&[u64::MAX]).unwrap().total(), u64::MAX);
}

#[test]
fn categorical_ticket_boundaries_own_their_category() {
    let weights = [0u64, 2, 5];
    let category = Categorical::new(&weights).unwrap();

    // The interval is `[0, 7)`, and a 64-bit word is rejected only below `2^64 mod 7`, which is 2.
    for (word, expected) in [(7u64, 1), (8, 1), (9, 2), (13, 2)] {
        let ticket = word % 7;
        let mut queue = ByteQueue::of_u64(&[word]);
        let sampled = category.sample(&mut queue).unwrap();
        assert_eq!(sampled, model_category(&weights, ticket));
        assert_eq!(sampled, expected, "ticket {ticket} did not own category {expected}");
        assert_eq!(queue.position, 8);
    }
}

#[test]
fn categorical_never_selects_a_zero_weight_category() {
    let weights = [0u64, 2, 5];
    let category = Categorical::new(&weights).unwrap();
    let mut random = Random::new(ChaCha12::from_u64(3, 4));
    for _ in 0..512 {
        let index = category.sample(&mut random).unwrap();
        assert!(index == 1 || index == 2, "a zero weight category was selected: {index}");
    }
}

#[test]
fn categorical_with_a_single_unit_ticket_consumes_nothing() {
    let weights = [0u64, 0, 1];
    let category = Categorical::new(&weights).unwrap();
    assert_eq!(category.total(), 1);
    let mut random = fresh();
    assert_eq!(category.sample(&mut random).unwrap(), 2);
    assert_eq!(probe(&mut random), probe(&mut fresh()));
}

#[test]
fn categorical_reports_source_failures() {
    let weights = [1u64, 1];
    let category = Categorical::new(&weights).unwrap();
    assert_eq!(category.sample(&mut FailingSource).unwrap_err(), RandomError::Source(SourceError::Failed));
}
