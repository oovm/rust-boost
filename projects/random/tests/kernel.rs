//! Contract tests for the source-generic sampling kernel.
//!
//! The kernel is exercised through the public API only. A deterministic byte queue plays an exact
//! word sequence so the tests can assert both the sampled value and how many bytes were consumed,
//! and the same entry points are driven by `Stream`, by a custom `RandomSource`, and by `Secure`,
//! which is the architectural point of the shared kernel.

use random::{EntropySource, Error, ErrorKind, RandomSource, RangeEndpoint, Secure, Seed, Stream};

/// Deterministic source that plays a fixed byte sequence and reports exhaustion at the end.
struct ByteQueue {
    bytes: Vec<u8>,
    position: usize,
}

impl ByteQueue {
    /// Builds a queue from little-endian `u32` words.
    fn of_u32(words: &[u32]) -> Self {
        let bytes = words.iter().flat_map(|word| word.to_le_bytes()).collect();
        Self { bytes, position: 0 }
    }

    /// Builds a queue from little-endian `u64` words.
    fn of_u64(words: &[u64]) -> Self {
        let bytes = words.iter().flat_map(|word| word.to_le_bytes()).collect();
        Self { bytes, position: 0 }
    }
}

impl RandomSource for ByteQueue {
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Error> {
        let end = self.position + dst.len();
        if end > self.bytes.len() {
            return Err(Error::new(ErrorKind::StreamExhausted));
        }
        dst.copy_from_slice(&self.bytes[self.position..end]);
        self.position = end;
        Ok(())
    }
}

/// Source that always fails, to check that no sample is invented on failure.
struct FailingSource;

impl RandomSource for FailingSource {
    fn try_fill_bytes(&mut self, _dst: &mut [u8]) -> Result<(), Error> {
        Err(Error::new(ErrorKind::BackendFailure))
    }
}

/// Reference model of the documented rejection mapping for a 32-bit word.
///
/// It returns `None` when the word is rejected, so the caller walks the word list exactly like the
/// rejection loop does.
fn model_u32(word: u32, start: u32, end: u32) -> Option<u32> {
    if end < start {
        return None;
    }
    if start == end {
        return Some(start);
    }
    let interval = end.wrapping_sub(start).wrapping_add(1);
    if interval == 0 {
        return Some(word);
    }
    let threshold = interval.wrapping_neg() % interval;
    if word < threshold {
        return None;
    }
    Some(start.wrapping_add(word % interval))
}

/// Returns the first accepted value and how many words the model consumed.
fn model_first_accepted(words: &[u32], start: u32, end: u32) -> Option<(u32, usize)> {
    for (index, word) in words.iter().enumerate() {
        if let Some(value) = model_u32(*word, start, end) {
            return Some((value, index + 1));
        }
    }
    None
}

#[test]
fn rejection_path_follows_the_documented_mapping() {
    let words = [0u32, 5, 9];
    let mut queue = ByteQueue::of_u32(&words);
    let sampled = u32::sample_bounded(&mut queue, 1, 3).unwrap();

    let (expected, consumed) = model_first_accepted(&words, 1, 3).unwrap();
    assert_eq!(sampled, expected);
    assert_eq!(queue.position, consumed * 4);
}

#[test]
fn full_domain_consumes_exactly_one_word() {
    let mut queue = ByteQueue::of_u32(&[0xdead_beef, 1]);
    assert_eq!(u32::sample_bounded(&mut queue, 0, u32::MAX).unwrap(), 0xdead_beef);
    assert_eq!(queue.position, 4);

    let mut queue = ByteQueue::of_u64(&[0x0123_4567_89ab_cdef, 1]);
    assert_eq!(u64::sample_bounded(&mut queue, 0, u64::MAX).unwrap(), 0x0123_4567_89ab_cdef);
    assert_eq!(queue.position, 8);
}

#[test]
fn single_value_interval_consumes_nothing() {
    let mut queue = ByteQueue::of_u32(&[7]);
    assert_eq!(u32::sample_bounded(&mut queue, 7, 7).unwrap(), 7);
    assert_eq!(queue.position, 0);
}

#[test]
fn invalid_bounds_fail_before_reading_any_byte() {
    let mut queue = ByteQueue::of_u32(&[0]);
    let error = u32::sample_bounded(&mut queue, 9, 3).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::InvalidRange);
    assert_eq!(queue.position, 0);
}

#[test]
fn signed_endpoints_use_the_order_preserving_transform() {
    let mut queue = ByteQueue::of_u32(&[0x8000_0000, 0]);
    assert_eq!(i32::sample_bounded(&mut queue, i32::MIN, i32::MAX).unwrap(), 0);
    assert_eq!(queue.position, 4);

    let words = [0u32, 0x7fff_fffe, 0];
    let mut queue = ByteQueue::of_u32(&words);
    let sampled = i32::sample_bounded(&mut queue, -2, 0).unwrap();

    let ordered_start = (-2i32 as u32) ^ 0x8000_0000;
    let ordered_end = (0i32 as u32) ^ 0x8000_0000;
    let (expected, consumed) = model_first_accepted(&words, ordered_start, ordered_end).unwrap();
    assert_eq!(sampled, (expected ^ 0x8000_0000) as i32);
    assert_eq!(queue.position, consumed * 4);

    let mut queue = ByteQueue::of_u64(&[0x8000_0000_0000_0000, 0]);
    assert_eq!(i64::sample_bounded(&mut queue, i64::MIN, i64::MAX).unwrap(), 0);
    assert_eq!(queue.position, 8);
}

#[test]
fn source_failure_propagates_without_inventing_a_sample() {
    let error = u32::sample_bounded(&mut FailingSource, 1, 6).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::BackendFailure);
    assert_eq!(error.platform_code(), None);
}

/// The stream and the shared kernel must agree on both value and consumption.
#[test]
fn stream_range_and_the_shared_kernel_agree() {
    let mut through_range = Stream::new(Seed::from_u64(42), 0);
    let mut through_kernel = Stream::new(Seed::from_u64(42), 0);
    for _ in 0..64 {
        assert_eq!(through_range.range(1u32..=6).unwrap(), u32::sample_bounded(&mut through_kernel, 1, 6).unwrap());
        assert_eq!(through_range.range(-4i32..=4).unwrap(), i32::sample_bounded(&mut through_kernel, -4, 4).unwrap());
        assert_eq!(through_range.range(10u64..40).unwrap(), u64::sample_bounded(&mut through_kernel, 10, 39).unwrap());
        assert_eq!(
            through_range.range(i64::MIN..=-1).unwrap(),
            i64::sample_bounded(&mut through_kernel, i64::MIN, -1).unwrap()
        );
    }
    assert_eq!(through_range.u32().unwrap(), through_kernel.u32().unwrap());
}

/// Deterministic provider that plays every byte of a fixed pattern.
struct PatternProvider(u8);

impl EntropySource for PatternProvider {
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Error> {
        dst.fill(self.0);
        Ok(())
    }
}

/// Provider that always fails, to check that a handle keeps its clearing contract when sampled.
struct FailingProvider;

impl EntropySource for FailingProvider {
    fn try_fill_bytes(&mut self, _dst: &mut [u8]) -> Result<(), Error> {
        Err(Error::new(ErrorKind::EntropyUnavailable))
    }
}

#[test]
fn secure_handle_drives_the_shared_kernel() {
    let mut secure = Secure::with_entropy(PatternProvider(0x42));
    let word = u32::from_le_bytes([0x42, 0x42, 0x42, 0x42]);
    let (expected, _) = model_first_accepted(&[word], 1, 3).unwrap();
    assert_eq!(u32::sample_bounded(&mut secure, 1, 3).unwrap(), expected);

    let mut secure = Secure::with_entropy(PatternProvider(0xff));
    assert_eq!(u32::sample_bounded(&mut secure, 0, u32::MAX).unwrap(), u32::MAX);
}

#[test]
fn secure_sampling_reports_the_provider_failure() {
    let mut secure = Secure::with_entropy(FailingProvider);
    let error = u32::sample_bounded(&mut secure, 1, 6).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::EntropyUnavailable);
}
