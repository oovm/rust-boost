use crate::{
    chacha::ChaChaCore,
    error::{Error, ErrorKind},
    range::{RangeEndpoint, SampleRange},
    seed::Seed,
    source::RandomSource,
};

/// Deterministic random stream of the `boost-chacha12-v1` profile.
///
/// A stream is defined by a [`Seed`] and a logical stream identifier. It never reads host entropy,
/// never depends on thread scheduling or target word size, and consumes its state through
/// `&mut self` only.
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
    pub fn fill(&mut self, dst: &mut [u8]) -> Result<(), Error> {
        self.core.fill(dst)
    }

    /// Reads the next 4 bytes as a little-endian `u32`.
    pub fn u32(&mut self) -> Result<u32, Error> {
        let mut bytes = [0u8; 4];
        self.core.fill(&mut bytes)?;
        Ok(u32::from_le_bytes(bytes))
    }

    /// Reads the next 8 bytes as a little-endian `u64`.
    pub fn u64(&mut self) -> Result<u64, Error> {
        let mut bytes = [0u8; 8];
        self.core.fill(&mut bytes)?;
        Ok(u64::from_le_bytes(bytes))
    }

    /// Samples an unbiased value from an integer range.
    ///
    /// Bounds are validated before any byte is consumed, and the interval is sampled with
    /// rejection sampling. The sampled value only depends on the interval, never on the target
    /// word size.
    pub fn range<R, B>(&mut self, bounds: B) -> Result<R, Error>
    where
        R: RangeEndpoint,
        B: SampleRange<R>,
    {
        let (start, end) = bounds.bounds()?;
        R::sample_bounded(self, start, end)
    }

    /// Samples a uniform `f32` in `[0, 1)` from the top 24 bits of one word.
    pub fn unit_f32(&mut self) -> Result<f32, Error> {
        Ok((self.u32()? >> 8) as f32 * (1.0 / 16_777_216.0))
    }

    /// Samples a uniform `f64` in `[0, 1)` from the top 53 bits of one word.
    pub fn unit_f64(&mut self) -> Result<f64, Error> {
        Ok((self.u64()? >> 11) as f64 * (1.0 / 9_007_199_254_740_992.0))
    }

    /// Decides the rational probability `numerator / denominator`.
    ///
    /// A zero or full probability returns without consuming bytes. Otherwise the stream samples an
    /// integer in `[0, denominator)` and compares it with `numerator`.
    pub fn chance(&mut self, numerator: u64, denominator: u64) -> Result<bool, Error> {
        if denominator == 0 || numerator > denominator {
            return Err(Error::new(ErrorKind::InvalidProbability));
        }
        if numerator == 0 {
            return Ok(false);
        }
        if numerator == denominator {
            return Ok(true);
        }
        Ok(self.range(0u64..denominator)? < numerator)
    }

    /// Samples a uniform index in `0..len`.
    ///
    /// An empty collection returns `None` without consuming bytes. The index is always sampled as a
    /// `u64` interval, so the same collection length yields the same index on every target.
    pub fn index(&mut self, len: usize) -> Result<Option<usize>, Error> {
        if len == 0 {
            return Ok(None);
        }
        let bound = u64::try_from(len).map_err(|_| Error::new(ErrorKind::LengthOverflow))?;
        let sampled = self.range(0u64..bound)?;
        let index = usize::try_from(sampled).map_err(|_| Error::new(ErrorKind::LengthOverflow))?;
        Ok(Some(index))
    }

    /// Samples a reference to one element, or `None` for an empty slice.
    pub fn choose<'a, T>(&mut self, items: &'a [T]) -> Result<Option<&'a T>, Error> {
        match self.index(items.len())? {
            Some(index) => Ok(Some(&items[index])),
            None => Ok(None),
        }
    }

    /// Shuffles `items` with descending Fisher-Yates.
    ///
    /// Each step samples an index in `0..i + 1` through the `u64` index path. A slice of length 0
    /// or 1 is left untouched without consuming bytes. If the stream fails mid-way, the swaps
    /// already performed are kept and the error is returned.
    pub fn shuffle<T>(&mut self, items: &mut [T]) -> Result<(), Error> {
        for i in (1..items.len()).rev() {
            let Some(target) = self.index(i + 1)?
            else {
                return Err(Error::new(ErrorKind::InvalidRange));
            };
            items.swap(i, target);
        }
        Ok(())
    }
}

impl RandomSource for Stream {
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Error> {
        self.fill(dst)
    }
}
