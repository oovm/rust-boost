//! Contract tests for the sampling surface shared by `Random<ChaCha12>` and `Random<Secure>`.
//!
//! `Random` exposes the same first-batch entries whatever source it wraps. The tests check the
//! shared mapping by replaying one deterministic keystream into an entropy provider: the provider is
//! not the platform backend, but it is a real `EntropySource`, so the handle consumes the same bytes
//! and must reach the same values, the same consumption, and the same end position as the
//! deterministic source.
//!
//! The tests observe the host only through a recording or failing provider, so the clearing
//! contract and the "no request for a degenerate call" contract are verified from the outside.

use core::cell::Cell;

use random::{ChaCha12, EntropyError, EntropySource, Random, RandomError, Secure, SourceError};

/// Provider that replays a fixed byte sequence and reports failure at the end.
struct ReplayProvider {
    bytes: Vec<u8>,
    position: usize,
}

impl EntropySource for ReplayProvider {
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), EntropyError> {
        let end = self.position + dst.len();
        if end > self.bytes.len() {
            return Err(EntropyError::Failed);
        }
        dst.copy_from_slice(&self.bytes[self.position..end]);
        self.position = end;
        Ok(())
    }
}

/// Provider that records how often it was asked for bytes.
struct CountingProvider<'a> {
    calls: &'a Cell<usize>,
}

impl EntropySource for CountingProvider<'_> {
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), EntropyError> {
        self.calls.set(self.calls.get() + 1);
        dst.fill(0x42);
        Ok(())
    }
}

/// Provider that always fails.
struct FailingProvider;

impl EntropySource for FailingProvider {
    fn try_fill_bytes(&mut self, _dst: &mut [u8]) -> Result<(), EntropyError> {
        Err(EntropyError::Unavailable)
    }
}

/// Provider that serves one exact `u64` word and then fails.
struct OneWordThenFail {
    word: u64,
    used: bool,
}

impl EntropySource for OneWordThenFail {
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), EntropyError> {
        if self.used || dst.len() != 8 {
            return Err(EntropyError::Unavailable);
        }
        self.used = true;
        dst.copy_from_slice(&self.word.to_le_bytes());
        Ok(())
    }
}

/// Returns the first `length` bytes of one deterministic source, so a provider can replay them.
fn keystream(length: usize) -> Vec<u8> {
    let mut random = Random::new(ChaCha12::from_u64(42, 0));
    let mut bytes = vec![0u8; length];
    random.fill(&mut bytes).unwrap();
    bytes
}

/// The failure reported by a source whose provider is unavailable, wrapped as one sampling error.
fn unavailable() -> RandomError {
    RandomError::Source(SourceError::Entropy(EntropyError::Unavailable))
}

#[test]
fn secure_replays_a_stream_byte_for_byte() {
    let bytes = keystream(8192);
    let mut secure = Random::new(Secure::with_entropy(ReplayProvider { bytes, position: 0 }));
    let mut stream = Random::new(ChaCha12::from_u64(42, 0));

    let rotations = [10, 20, 30];
    for _ in 0..32 {
        assert_eq!(secure.u32().unwrap(), stream.u32().unwrap());
        assert_eq!(secure.u64().unwrap(), stream.u64().unwrap());
        assert_eq!(secure.range_u32(1..=6).unwrap(), stream.range_u32(1..=6).unwrap());
        assert_eq!(secure.range_i32(-4..=4).unwrap(), stream.range_i32(-4..=4).unwrap());
        assert_eq!(secure.unit_f32().unwrap().to_bits(), stream.unit_f32().unwrap().to_bits());
        assert_eq!(secure.unit_f64().unwrap().to_bits(), stream.unit_f64().unwrap().to_bits());
        assert_eq!(secure.chance(1, 3).unwrap(), stream.chance(1, 3).unwrap());
        assert_eq!(secure.index(97).unwrap(), stream.index(97).unwrap());
        assert_eq!(secure.choose(&rotations).unwrap(), stream.choose(&rotations).unwrap());
    }

    let mut secure_items: Vec<u32> = (0..32).collect();
    let mut stream_items: Vec<u32> = (0..32).collect();
    secure.shuffle(&mut secure_items).unwrap();
    stream.shuffle(&mut stream_items).unwrap();
    assert_eq!(secure_items, stream_items);

    // The end position must match too, so neither side consumed a word the other skipped.
    assert_eq!(secure.u32().unwrap(), stream.u32().unwrap());
}

#[test]
fn degenerate_calls_never_touch_the_host() {
    let calls = Cell::new(0);
    let mut secure = Random::new(Secure::with_entropy(CountingProvider { calls: &calls }));

    assert_eq!(secure.range_u32(7..=7).unwrap(), 7);
    assert!(!secure.chance(0, 5).unwrap());
    assert!(secure.chance(5, 5).unwrap());
    assert_eq!(secure.index(0).unwrap(), None);
    assert_eq!(secure.index(1).unwrap(), Some(0));
    let empty: [u8; 0] = [];
    assert_eq!(secure.choose(&empty).unwrap(), None);
    let mut nothing: [u8; 0] = [];
    secure.shuffle(&mut nothing).unwrap();
    let mut single = [7u8];
    secure.shuffle(&mut single).unwrap();
    assert_eq!(single, [7]);

    secure.fill(&mut []).unwrap();
    assert_eq!(calls.get(), 0);
}

#[test]
// The reversed bounds are the subject of this test, not a mistake in an iteration.
#[allow(clippy::reversed_empty_ranges)]
fn invalid_arguments_fail_before_touching_the_host() {
    let calls = Cell::new(0);
    let mut secure = Random::new(Secure::with_entropy(CountingProvider { calls: &calls }));

    assert_eq!(secure.range_u32(9..3).unwrap_err(), RandomError::InvalidRange);
    assert_eq!(secure.chance(6, 5).unwrap_err(), RandomError::InvalidProbability);
    assert_eq!(secure.chance(1, 0).unwrap_err(), RandomError::InvalidProbability);
    assert_eq!(calls.get(), 0);
}

#[test]
fn a_failing_provider_is_reported_by_every_populated_entry() {
    let mut secure = Random::new(Secure::with_entropy(FailingProvider));

    assert_eq!(secure.u32().unwrap_err(), SourceError::Entropy(EntropyError::Unavailable));
    assert_eq!(secure.u64().unwrap_err(), SourceError::Entropy(EntropyError::Unavailable));
    assert_eq!(secure.range_u32(1..=6).unwrap_err(), unavailable());
    assert_eq!(secure.unit_f32().unwrap_err(), SourceError::Entropy(EntropyError::Unavailable));
    assert_eq!(secure.unit_f64().unwrap_err(), SourceError::Entropy(EntropyError::Unavailable));
    assert_eq!(secure.chance(1, 3).unwrap_err(), unavailable());
    assert_eq!(secure.index(4).unwrap_err(), unavailable());
    assert_eq!(secure.choose(&[1, 2]).unwrap_err(), unavailable());
    let mut items = [1, 2, 3];
    assert_eq!(secure.shuffle(&mut items).unwrap_err(), unavailable());
    assert_eq!(items, [1, 2, 3]);
}

#[test]
fn a_failing_provider_still_serves_the_degenerate_entries() {
    let mut secure = Random::new(Secure::with_entropy(FailingProvider));

    assert_eq!(secure.range_u32(3..=3).unwrap(), 3);
    assert!(!secure.chance(0, 1).unwrap());
    assert_eq!(secure.index(0).unwrap(), None);
    let mut single = [1u8];
    secure.shuffle(&mut single).unwrap();
    assert_eq!(single, [1]);
}

#[test]
fn a_mid_shuffle_failure_keeps_the_swaps_already_made() {
    // The first index request is answered with the exact word `3`, which maps to index `0` of the
    // `0..3` interval, so the descending Fisher-Yates pass swaps positions 2 and 0 before failing
    // on the next request.
    let mut secure = Random::new(Secure::with_entropy(OneWordThenFail { word: 3, used: false }));
    let mut items = [1, 2, 3];
    assert_eq!(secure.shuffle(&mut items).unwrap_err(), unavailable());
    assert_eq!(items, [3, 2, 1]);
}

#[test]
fn secure_entries_stay_inside_their_contract() {
    let bytes = keystream(8192);
    let mut secure = Random::new(Secure::with_entropy(ReplayProvider { bytes, position: 0 }));

    let choices = [10, 20, 30];
    for _ in 0..128 {
        let face = secure.range_u32(1..=6).unwrap();
        assert!((1..=6).contains(&face), "the range entry left its interval: {face}");

        let unit = secure.unit_f64().unwrap();
        assert!((0.0..1.0).contains(&unit), "the unit entry left [0, 1): {unit}");

        let single = secure.unit_f32().unwrap();
        assert!((0.0..1.0).contains(&single), "the unit entry left [0, 1): {single}");

        let choice = secure.choose(&choices).unwrap().unwrap();
        assert!(choices.contains(choice), "the choose entry returned a foreign element: {choice}");
    }

    let mut items: Vec<u32> = (0..16).collect();
    secure.shuffle(&mut items).unwrap();
    let mut sorted = items.clone();
    sorted.sort_unstable();
    assert_eq!(sorted, (0..16).collect::<Vec<u32>>());
    assert_ne!(items, (0..16).collect::<Vec<u32>>());
}
