//! Contract tests for the sampling surface of the `boost-chacha12-v1` profile.
//!
//! Several tests need to observe whether a call consumed bytes. They do so through the public API
//! only: a source that must not consume compares its next `u32` with the next `u32` of an identical
//! fresh source.

use random::{ChaCha12, Random, RandomError};

/// Source used by tests that only need a stable identity.
fn fresh() -> Random<ChaCha12> {
    Random::new(ChaCha12::from_u64(0x5eed, 3))
}

/// Reads one word to detect whether previous calls consumed bytes.
fn probe(random: &mut Random<ChaCha12>) -> u32 {
    random.u32().unwrap()
}

#[test]
// The reversed and empty bounds are the subject of this test, not a mistake in an iteration.
#[allow(clippy::reversed_empty_ranges)]
fn reversed_and_empty_bounds_fail_without_consuming_bytes() {
    let mut random = fresh();
    assert_eq!(random.range_u32(9..3).unwrap_err(), RandomError::InvalidRange);
    assert_eq!(random.range_u32(3..=2).unwrap_err(), RandomError::InvalidRange);
    assert_eq!(random.range_u32(0..0).unwrap_err(), RandomError::InvalidRange);
    assert_eq!(random.range_i32(4..=-4).unwrap_err(), RandomError::InvalidRange);
    assert_eq!(random.range_u64(6..=5).unwrap_err(), RandomError::InvalidRange);
    assert_eq!(random.range_i64(0..0).unwrap_err(), RandomError::InvalidRange);
    assert_eq!(probe(&mut random), probe(&mut fresh()));
}

#[test]
fn single_value_interval_consumes_nothing() {
    let mut random = fresh();
    assert_eq!(random.range_u32(7..=7).unwrap(), 7);
    assert_eq!(random.range_i32(-5..=-5).unwrap(), -5);
    assert_eq!(random.range_u64(0..1).unwrap(), 0);
    assert_eq!(random.range_i64(i64::MIN..=i64::MIN).unwrap(), i64::MIN);
    assert_eq!(probe(&mut random), probe(&mut fresh()));
}

#[test]
fn ranges_stay_inside_their_bounds() {
    let mut random = fresh();
    for _ in 0..512 {
        let signed = random.range_i32(-4..=4).unwrap();
        assert!((-4..=4).contains(&signed));
        let unsigned = random.range_u64(10..40).unwrap();
        assert!((10..40).contains(&unsigned));
        let wide = random.range_u32(1..=6).unwrap();
        assert!((1..=6).contains(&wide));
    }
}

#[test]
fn full_domain_ranges_consume_exactly_one_word() {
    let mut random = fresh();
    random.range_u32(0..=u32::MAX).unwrap();
    let mut reference = fresh();
    reference.u32().unwrap();
    assert_eq!(probe(&mut random), probe(&mut reference));

    let mut random = fresh();
    random.range_u64(0..=u64::MAX).unwrap();
    let mut reference = fresh();
    reference.u64().unwrap();
    assert_eq!(random.u64().unwrap(), reference.u64().unwrap());

    let mut random = fresh();
    random.range_i32(i32::MIN..=i32::MAX).unwrap();
    let mut reference = fresh();
    reference.u32().unwrap();
    assert_eq!(random.u32().unwrap(), reference.u32().unwrap());
}

#[test]
fn range_width_follows_the_unsigned_representation() {
    let mut random = fresh();
    random.range_i32(-1..=0).unwrap();
    let mut reference = fresh();
    reference.u32().unwrap();
    assert_eq!(probe(&mut random), probe(&mut reference));

    let mut random = fresh();
    random.range_i64(i64::MIN..=-1).unwrap();
    let mut reference = fresh();
    reference.u64().unwrap();
    assert_eq!(random.u64().unwrap(), reference.u64().unwrap());
}

#[test]
fn chance_boundaries_consume_nothing() {
    let mut random = fresh();
    assert!(!random.chance(0, 5).unwrap());
    assert!(random.chance(5, 5).unwrap());
    assert_eq!(probe(&mut random), probe(&mut fresh()));
}

#[test]
fn chance_rejects_invalid_probabilities() {
    let mut random = fresh();
    assert_eq!(random.chance(1, 0).unwrap_err(), RandomError::InvalidProbability);
    assert_eq!(random.chance(6, 5).unwrap_err(), RandomError::InvalidProbability);
    assert_eq!(probe(&mut random), probe(&mut fresh()));
}

#[test]
fn chance_produces_both_outcomes() {
    let mut random = fresh();
    let mut outcomes = [0usize; 2];
    for _ in 0..256 {
        outcomes[usize::from(random.chance(1, 3).unwrap())] += 1;
    }
    assert!(outcomes[0] > 0 && outcomes[1] > 0, "chance(1, 3) never varied: {outcomes:?}");
}

#[test]
fn empty_and_single_index_consume_nothing() {
    let mut random = fresh();
    assert_eq!(random.index(0).unwrap(), None);
    assert_eq!(random.index(1).unwrap(), Some(0));
    assert_eq!(probe(&mut random), probe(&mut fresh()));
}

#[test]
fn index_stays_below_the_length() {
    let mut random = fresh();
    for length in [2usize, 3, 10, 1024] {
        for _ in 0..64 {
            let index = random.index(length).unwrap().unwrap();
            assert!(index < length);
        }
    }
}

#[test]
fn choose_handles_empty_and_populated_slices() {
    let empty: [i32; 0] = [];
    let mut random = fresh();
    assert_eq!(random.choose(&empty).unwrap(), None);
    assert_eq!(probe(&mut random), probe(&mut fresh()));

    let rotations = [-90, 0, 90];
    let mut random = fresh();
    for _ in 0..64 {
        let chosen = random.choose(&rotations).unwrap().unwrap();
        assert!(rotations.contains(chosen));
    }
}

#[test]
fn short_shuffles_consume_nothing() {
    let mut random = fresh();
    random.shuffle(&mut [] as &mut [u8]).unwrap();
    let mut single = [7u8];
    random.shuffle(&mut single).unwrap();
    assert_eq!(single, [7]);
    assert_eq!(probe(&mut random), probe(&mut fresh()));
}

#[test]
fn shuffle_is_a_permutation() {
    let mut items: Vec<u32> = (0..64).collect();
    fresh().shuffle(&mut items).unwrap();
    let mut sorted = items.clone();
    sorted.sort_unstable();
    assert_eq!(sorted, (0..64).collect::<Vec<u32>>());
    assert_ne!(items, (0..64).collect::<Vec<u32>>());
}

#[test]
fn shuffle_is_reproducible() {
    let mut left: Vec<u32> = (0..32).collect();
    let mut right: Vec<u32> = (0..32).collect();
    Random::new(ChaCha12::from_u64(4, 2)).shuffle(&mut left).unwrap();
    Random::new(ChaCha12::from_u64(4, 2)).shuffle(&mut right).unwrap();
    assert_eq!(left, right);
}

#[test]
fn unit_interval_floats_stay_in_range() {
    let mut random = fresh();
    for _ in 0..512 {
        let single = random.unit_f32().unwrap();
        assert!((0.0..1.0).contains(&single), "unit_f32 returned {single}");
        let double = random.unit_f64().unwrap();
        assert!((0.0..1.0).contains(&double), "unit_f64 returned {double}");
    }
}

/// Profile-local golden values recorded by this crate: `ChaCha12::from_u64(42, 0)`.
///
/// They lock the float construction and the integer mapping of the profile. They were recorded
/// after the external ChaCha12 vectors passed, so they detect changes rather than prove correctness
/// on their own.
#[test]
fn sampling_goldens_are_locked() {
    let mut random = Random::new(ChaCha12::from_u64(42, 0));
    assert_eq!(random.unit_f32().unwrap().to_bits(), 0x3f3c_ac71);

    let mut random = Random::new(ChaCha12::from_u64(42, 0));
    assert_eq!(random.unit_f64().unwrap().to_bits(), 0x3fdb_a660_942f_2b1c);

    let mut random = Random::new(ChaCha12::from_u64(42, 0));
    assert_eq!(random.range_u32(0..1000).unwrap(), 837);
}
