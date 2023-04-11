use crate::{
    chacha::ChaChaCore,
    error::RandomError,
    range::{RangeEndpoint, SampleRange},
    sampling,
    seed::Seed,
    source::RandomSource,
};

/// Deterministic random stream of the `boost-chacha12-v1` profile.
///
/// A stream is defined by a [`Seed`] and a logical stream identifier. It never reads host entropy,
/// never depends on thread scheduling or target word size, and consumes its state through
/// `&mut self` only.
///
/// The sampling entries below are shared with [`Secure`](crate::Secure): both types delegate to the
/// same functions, so one profile mapping and one consumption order serve both.
pub struct Stream {
    core: ChaChaCore,
}

impl Stream {
    /// Frozen output profile implemented by this type.
    ///
    /// The profile fixes the algorithm, seed encoding, byte consumption order, integer sampling,
    /// float construction, and shuffle order as one contract.
    pub const PROFILE: &str = "boost-chacha12-v1";

    /// Creates a deterministic stream from `seed` and a logical `stream_id`.
    ///
    /// `stream_id` distinguishes the streams of one seed. It must come from a stable logical task
    /// identity, not from a thread number, a worker count, or a completion order.
    pub fn new(seed: Seed, stream_id: u64) -> Self {
        Self { core: ChaChaCore::new(&seed.to_bytes(), stream_id) }
    }

    /// Fills `dst` with the next bytes of the stream.
    ///
    /// A request longer than the remaining profile capacity fails without advancing the stream and
    /// clears `dst`.
    pub fn fill(&mut self, dst: &mut [u8]) -> Result<(), RandomError> {
        self.core.fill(dst)
    }

    /// Reads the next 4 bytes as a little-endian `u32`.
    pub fn u32(&mut self) -> Result<u32, RandomError> {
        sampling::read_u32(self)
    }

    /// Reads the next 8 bytes as a little-endian `u64`.
    pub fn u64(&mut self) -> Result<u64, RandomError> {
        sampling::read_u64(self)
    }

    /// Samples an unbiased value from an integer range.
    ///
    /// Bounds are validated before any byte is consumed, and the interval is sampled with
    /// rejection sampling. The sampled value only depends on the interval, never on the target
    /// word size.
    pub fn range<R, B>(&mut self, bounds: B) -> Result<R, RandomError>
    where
        R: RangeEndpoint,
        B: SampleRange<R>,
    {
        sampling::range(self, bounds)
    }

    /// Samples a uniform `f32` in `[0, 1)` from the top 24 bits of one word.
    pub fn unit_f32(&mut self) -> Result<f32, RandomError> {
        sampling::unit_f32(self)
    }

    /// Samples a uniform `f64` in `[0, 1)` from the top 53 bits of one word.
    pub fn unit_f64(&mut self) -> Result<f64, RandomError> {
        sampling::unit_f64(self)
    }

    /// Decides the rational probability `numerator / denominator`.
    ///
    /// A zero or full probability returns without consuming bytes. Otherwise the stream samples an
    /// integer in `[0, denominator)` and compares it with `numerator`.
    pub fn chance(&mut self, numerator: u64, denominator: u64) -> Result<bool, RandomError> {
        sampling::chance(self, numerator, denominator)
    }

    /// Samples a uniform index in `0..len`.
    ///
    /// An empty collection returns `None` without consuming bytes. The index is always sampled as a
    /// `u64` interval, so the same collection length yields the same index on every target.
    pub fn index(&mut self, len: usize) -> Result<Option<usize>, RandomError> {
        sampling::index(self, len)
    }

    /// Samples a reference to one element, or `None` for an empty slice.
    pub fn choose<'a, T>(&mut self, items: &'a [T]) -> Result<Option<&'a T>, RandomError> {
        sampling::choose(self, items)
    }

    /// Shuffles `items` with descending Fisher-Yates.
    ///
    /// Each step samples an index in `0..i + 1` through the `u64` index path. A slice of length 0
    /// or 1 is left untouched without consuming bytes. If the stream fails mid-way, the swaps
    /// already performed are kept and the error is returned.
    pub fn shuffle<T>(&mut self, items: &mut [T]) -> Result<(), RandomError> {
        sampling::shuffle(self, items)
    }
}

impl RandomSource for Stream {
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), RandomError> {
        self.fill(dst)
    }
}
