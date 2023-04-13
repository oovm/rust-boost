//! Verification of the sampling and consumption contract of §9.2.
//!
//! The sampling kernel is checked against a programmed word sequence instead of a statistical
//! observation: a caller-owned `RandomSource` plays exactly the words the test wants, so the test
//! knows what the documented rejection mapping must return, how many words it must consume, and
//! whether every residue of an interval is reached the same number of times inside a complete set
//! of cycles. A frequency check alone would not distinguish an exact mapping from a biased one.

use random::{ChaCha12, Random, RandomSource, SourceError, distribution::Uniform};

/// Deterministic source that plays a programmed sequence of whole words.
struct WordQueue {
    bytes: Vec<u8>,
    position: usize,
}

impl WordQueue {
    /// Programs a sequence of little-endian `u32` words.
    fn of_u32(words: &[u32]) -> Self {
        let bytes = words.iter().flat_map(|word| word.to_le_bytes()).collect();
        Self { bytes, position: 0 }
    }

    /// Programs a sequence of little-endian `u64` words.
    fn of_u64(words: &[u64]) -> Self {
        let bytes = words.iter().flat_map(|word| word.to_le_bytes()).collect();
        Self { bytes, position: 0 }
    }

    /// Programs `count` consecutive `u32` words starting at `first`.
    fn consecutive_u32(first: u32, count: u64) -> Self {
        let words: Vec<u32> = (0..count).map(|step| first.wrapping_add(step as u32)).collect();
        Self::of_u32(&words)
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

/// Returns the documented rejection threshold `2^w mod n` for an interval of `n` values.
fn threshold_u32(interval: u32) -> u32 {
    interval.wrapping_neg() % interval
}

/// Returns the documented rejection threshold for a 64-bit interval of `n` values.
fn threshold_u64(interval: u64) -> u64 {
    interval.wrapping_neg() % interval
}

/// Counts how often each value of `start..=end` is sampled across complete cycles of the accepted
/// word window, and returns the per-value counts.
fn cycle_counts_u32(start: u32, end: u32, cycles: u64) -> (Vec<u64>, u64) {
    let interval = end.wrapping_sub(start).wrapping_add(1);
    let first = threshold_u32(interval);
    let window = u64::from(interval) * cycles;
    let mut queue = WordQueue::consecutive_u32(first, window);
    let distribution = Uniform::<u32>::new(start..=end).unwrap();
    let mut counts = vec![0u64; interval as usize];
    for _ in 0..window {
        let value = distribution.sample(&mut queue).expect("accepted word window");
        counts[(value - start) as usize] += 1;
    }
    (counts, queue.position as u64)
}

#[test]
fn complete_word_cycles_are_exactly_uniform() {
    // A power-of-two interval, a small non-power-of-two interval, and an interval whose threshold
    // is far from zero all have to reach every value exactly the same number of times.
    for (start, end, cycles) in [(0u32, 3u32, 100u64), (1, 6, 100), (0, 999, 10), (10, 10 + 255, 32)] {
        let (counts, consumed) = cycle_counts_u32(start, end, cycles);
        let interval = u64::from(end - start + 1);
        assert!(counts.iter().all(|count| *count == cycles), "interval {start}..={end} was not uniform: {counts:?}");
        assert_eq!(consumed, interval * cycles * 4, "interval {start}..={end} consumed the wrong words");
    }
}

#[test]
fn complete_word_cycles_are_exactly_uniform_for_the_signed_domain() {
    // The signed endpoint passes through the order-preserving transform, so the same accepted-word
    // window must still reach every signed value the same number of times.
    let (start, end, cycles) = (-2i32, 2i32, 64u64);
    let interval = (end as i64 - start as i64 + 1) as u32;
    let first = threshold_u32(interval);
    let window = u64::from(interval) * cycles;
    let mut queue = WordQueue::consecutive_u32(first, window);
    let distribution = Uniform::<i32>::new(start..=end).unwrap();
    let mut counts = vec![0u64; interval as usize];
    for _ in 0..window {
        let value = distribution.sample(&mut queue).unwrap();
        counts[(value as i64 - start as i64) as usize] += 1;
    }
    assert!(counts.iter().all(|count| *count == cycles), "signed interval was not uniform: {counts:?}");
}

#[test]
fn complete_word_cycles_are_exactly_uniform_for_the_wide_endpoints() {
    let (start, end, cycles) = (1u64, 7u64, 64u64);
    let interval = end - start + 1;
    let first = threshold_u64(interval);
    let window = interval * cycles;
    let mut queue = WordQueue::of_u64(&(0..window).map(|step| first + step).collect::<Vec<u64>>());
    let distribution = Uniform::<u64>::new(start..=end).unwrap();
    let mut counts = vec![0u64; interval as usize];
    for _ in 0..window {
        let value = distribution.sample(&mut queue).unwrap();
        counts[(value - start) as usize] += 1;
    }
    assert!(counts.iter().all(|count| *count == cycles), "wide interval was not uniform: {counts:?}");
    assert_eq!(queue.position as u64, interval * cycles * 8);

    let (start, end, cycles) = (-1i64, 1i64, 64u64);
    let interval = 3u64;
    let first = threshold_u64(interval);
    let window = interval * cycles;
    let mut queue = WordQueue::of_u64(&(0..window).map(|step| first + step).collect::<Vec<u64>>());
    let distribution = Uniform::<i64>::new(start..=end).unwrap();
    let mut counts = vec![0u64; interval as usize];
    for _ in 0..window {
        let value = distribution.sample(&mut queue).unwrap();
        counts[(value - start) as usize] += 1;
    }
    assert!(counts.iter().all(|count| *count == cycles), "wide signed interval was not uniform: {counts:?}");
}

#[test]
fn the_rejection_threshold_is_exactly_the_documented_remainder() {
    // The complete-cycle check only proves that the threshold is not above the window it starts
    // from. Feeding the word just below the documented threshold and the threshold word itself
    // pins it from both sides: a smaller threshold would accept the first word, a larger one would
    // reject the second.
    for (start, end) in [(0u32, 999u32), (1, 6), (0, 3), (10, 265)] {
        let interval = end - start + 1;
        let threshold = threshold_u32(interval);
        let distribution = Uniform::<u32>::new(start..=end).unwrap();

        if threshold > 0 {
            let mut queue = WordQueue::of_u32(&[threshold - 1, threshold]);
            let sampled = distribution.sample(&mut queue).unwrap();
            assert_eq!(sampled, start + (threshold % interval));
            assert_eq!(queue.position, 8, "interval {start}..={end} accepted a word below its threshold");
        }

        let mut queue = WordQueue::of_u32(&[threshold]);
        distribution.sample(&mut queue).unwrap();
        assert_eq!(queue.position, 4, "interval {start}..={end} rejected its threshold word");
    }

    let start = 0u64;
    let end = 999_999_999u64;
    let interval = end - start + 1;
    let threshold = threshold_u64(interval);
    let distribution = Uniform::<u64>::new(start..=end).unwrap();
    let mut queue = WordQueue::of_u64(&[threshold - 1, threshold]);
    assert_eq!(distribution.sample(&mut queue).unwrap(), start + (threshold % interval));
    assert_eq!(queue.position, 16, "the wide interval accepted a word below its threshold");

    let mut queue = WordQueue::of_u64(&[threshold]);
    distribution.sample(&mut queue).unwrap();
    assert_eq!(queue.position, 8, "the wide interval rejected its threshold word");
}

#[test]
fn rejection_runs_are_consumed_without_leaking_into_the_value() {
    // Interval 1..=3 of a 32-bit word has threshold 1, so the word `0` is rejected and every other
    // word is accepted. A run of rejected words must be consumed and the first accepted word alone
    // must decide the value.
    let words = [0u32, 0, 0, 0, 5, 0];
    let mut queue = WordQueue::of_u32(&words);
    let distribution = Uniform::<u32>::new(1..=3).unwrap();
    assert_eq!(distribution.sample(&mut queue).unwrap(), 1 + (5 % 3));
    assert_eq!(queue.position, 5 * 4, "the rejected run was not consumed word by word");

    // A wide interval with a large threshold rejects every word below it.
    let interval = u64::MAX;
    let first = threshold_u64(interval);
    assert!(first > 0, "the wide interval must keep a non-zero threshold");
    let mut words = vec![0u64; 4];
    words.extend([first, first + 1]);
    let mut queue = WordQueue::of_u64(&words);
    let distribution = Uniform::<u64>::new(0..=interval - 1).unwrap();
    assert_eq!(distribution.sample(&mut queue).unwrap(), first % interval);
    assert_eq!(queue.position, 5 * 8, "the rejected run was not consumed word by word");
}

#[test]
fn word_width_follows_the_endpoint_and_not_the_target() {
    // Each endpoint samples with its own unsigned width, so the same interval consumes four bytes on
    // the 32-bit endpoints and eight bytes on the 64-bit ones, whatever the host word size is.
    let mut queue = WordQueue::of_u64(&[0x0000_0007_0000_0003, 0]);
    assert_eq!(Uniform::<u32>::new(0..=255).unwrap().sample(&mut queue).unwrap(), 3);
    assert_eq!(queue.position, 4);

    let mut queue = WordQueue::of_u64(&[0x0000_0007_0000_0003, 0]);
    assert_eq!(Uniform::<u64>::new(0..=255).unwrap().sample(&mut queue).unwrap(), 3);
    assert_eq!(queue.position, 8);

    let mut queue = WordQueue::of_u64(&[0x0000_0007_0000_0003, 0]);
    assert_eq!(Uniform::<i32>::new(0..=255).unwrap().sample(&mut queue).unwrap(), 3);
    assert_eq!(queue.position, 4);
}

#[test]
fn full_domain_intervals_accept_every_word_of_their_width() {
    // A full-domain interval rejects nothing, so a threshold-free mapping consumes exactly one word
    // per sample for every endpoint type.
    let mut queue = WordQueue::of_u32(&[0xdead_beef, 0xffff_ffff]);
    let distribution = Uniform::<u32>::new(0..=u32::MAX).unwrap();
    assert_eq!(distribution.sample(&mut queue).unwrap(), 0xdead_beef);
    assert_eq!(distribution.sample(&mut queue).unwrap(), 0xffff_ffff);
    assert_eq!(queue.position, 8);

    let mut queue = WordQueue::of_u32(&[0xdead_beef, 0]);
    assert_eq!(
        Uniform::<i32>::new(i32::MIN..=i32::MAX).unwrap().sample(&mut queue).unwrap(),
        (0xdead_beef_u32 ^ 0x8000_0000) as i32
    );
    assert_eq!(queue.position, 4);

    let mut queue = WordQueue::of_u64(&[u64::MAX, 0]);
    assert_eq!(Uniform::<u64>::new(0..=u64::MAX).unwrap().sample(&mut queue).unwrap(), u64::MAX);
    assert_eq!(queue.position, 8);
}

#[test]
fn non_power_of_two_edge_intervals_stay_inside_their_bounds() {
    let mut random = Random::new(ChaCha12::from_u64(0xc0ffee, 2));

    for _ in 0..256 {
        let value = random.range_u32(0..u32::MAX).unwrap();
        assert!(value < u32::MAX, "0..u32::MAX returned {value}");

        let value = random.range_u32(1..=u32::MAX).unwrap();
        assert!(value >= 1, "1..=u32::MAX returned {value}");

        let value = random.range_i32(0..=i32::MAX).unwrap();
        assert!(value >= 0, "0..=i32::MAX returned {value}");

        let value = random.range_i32(i32::MIN..=-1).unwrap();
        assert!(value < 0, "i32::MIN..=-1 returned {value}");

        let value = random.range_u64(0..u64::MAX).unwrap();
        assert!(value < u64::MAX, "0..u64::MAX returned {value}");

        let value = random.range_i64(i64::MIN..=i64::MAX - 1).unwrap();
        assert!(value < i64::MAX, "i64::MIN..=i64::MAX-1 returned {value}");
    }
}

#[test]
fn every_fill_length_matches_one_reference() {
    let reference: Vec<u8> = {
        let mut random = Random::new(ChaCha12::from_u64(0x1234, 4));
        let mut bytes = vec![0u8; 129];
        random.fill(&mut bytes).unwrap();
        bytes
    };

    for length in [0usize, 1, 3, 4, 7, 8, 63, 64, 65, 127, 129] {
        let mut random = Random::new(ChaCha12::from_u64(0x1234, 4));
        let mut bytes = vec![0u8; length];
        random.fill(&mut bytes).unwrap();
        assert_eq!(bytes.as_slice(), &reference[..length], "fill({length}) diverged from the reference");
    }

    // A one-byte fill followed by a seven-byte fill is one eight-byte fill, so no block or word tail
    // is dropped between calls.
    let mut random = Random::new(ChaCha12::from_u64(0x1234, 4));
    let mut split = [0u8; 8];
    random.fill(&mut split[..1]).unwrap();
    random.fill(&mut split[1..]).unwrap();
    assert_eq!(split.as_slice(), &reference[..8]);

    // Scalar reads and fills share the byte order of the same position.
    let mut random = Random::new(ChaCha12::from_u64(0x1234, 4));
    assert_eq!(random.u32().unwrap(), u32::from_le_bytes(reference[..4].try_into().unwrap()));
    let mut middle = [0u8; 3];
    random.fill(&mut middle).unwrap();
    assert_eq!(middle.as_slice(), &reference[4..7]);
    assert_eq!(random.u64().unwrap(), u64::from_le_bytes(reference[7..15].try_into().unwrap()));
}

#[test]
fn large_fills_stay_contiguous_across_blocks() {
    const LENGTH: usize = 16 * 1024;
    let mut random = Random::new(ChaCha12::from_u64(0xabc, 7));
    let mut single = vec![0u8; LENGTH];
    random.fill(&mut single).unwrap();

    let mut random = Random::new(ChaCha12::from_u64(0xabc, 7));
    let mut chunked = vec![0u8; LENGTH];
    let mut written = 0;
    for length in [1usize, 63, 64, 65, 1000, 4096, 8192] {
        let end = (written + length).min(LENGTH);
        random.fill(&mut chunked[written..end]).unwrap();
        written = end;
    }
    random.fill(&mut chunked[written..]).unwrap();
    assert_eq!(chunked, single);
}
