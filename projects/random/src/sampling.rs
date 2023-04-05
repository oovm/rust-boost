//! Shared sampling entries of the `boost-chacha12-v1` profile.
//!
//! Every entry is written against [`RandomSource`], not against one concrete source type, so a
//! deterministic stream and a host entropy handle consume bytes through the same mapping,
//! rejection loop, and consumption order. Bytes are read as whole little-endian words of the
//! unsigned width of the endpoint.

use core::ops::{Add, Rem, Sub};

use crate::{
    error::{Error, ErrorKind},
    range::{RangeEndpoint, SampleRange},
    source::RandomSource,
};

/// Unsigned word width consumed by the profile for integer sampling.
pub(crate) trait Word: Copy + Ord + Add<Output = Self> + Sub<Output = Self> + Rem<Output = Self> {
    /// Zero of this width.
    const ZERO: Self;

    /// One of this width.
    const ONE: Self;

    /// Consumes one word of this width from an arbitrary source.
    fn next_word<S: RandomSource + ?Sized>(source: &mut S) -> Result<Self, Error>;

    /// Returns the two's complement negation of this value in this width.
    fn wrapping_neg(self) -> Self;

    /// Adds in this width without a debug overflow check.
    ///
    /// The interval of a full-domain range overflows on purpose, so the addition must wrap instead
    /// of panicking in a debug build.
    fn wrapping_add(self, other: Self) -> Self;
}

impl Word for u32 {
    const ZERO: Self = 0;
    const ONE: Self = 1;

    fn next_word<S: RandomSource + ?Sized>(source: &mut S) -> Result<Self, Error> {
        read_u32(source)
    }

    fn wrapping_neg(self) -> Self {
        0u32.wrapping_sub(self)
    }

    fn wrapping_add(self, other: Self) -> Self {
        u32::wrapping_add(self, other)
    }
}

impl Word for u64 {
    const ZERO: Self = 0;
    const ONE: Self = 1;

    fn next_word<S: RandomSource + ?Sized>(source: &mut S) -> Result<Self, Error> {
        read_u64(source)
    }

    fn wrapping_neg(self) -> Self {
        0u64.wrapping_sub(self)
    }

    fn wrapping_add(self, other: Self) -> Self {
        u64::wrapping_add(self, other)
    }
}

/// Samples an unbiased value from the inclusive ordered bounds `start..=end`.
///
/// An interval of a single value returns that value without consuming bytes. A full-domain
/// interval accepts every word, so it consumes exactly one word. Otherwise the value is obtained
/// by rejection sampling: the profile discards words below `2^w mod n` and then reduces modulo `n`,
/// which keeps every value equally likely.
pub(crate) fn sample_inclusive<W: Word, S: RandomSource + ?Sized>(source: &mut S, start: W, end: W) -> Result<W, Error> {
    if end < start {
        return Err(Error::new(ErrorKind::InvalidRange));
    }
    if start == end {
        return Ok(start);
    }

    let span = end - start;
    let interval = span.wrapping_add(W::ONE);
    if interval == W::ZERO {
        return W::next_word(source);
    }

    let threshold = interval.wrapping_neg() % interval;
    loop {
        let word = W::next_word(source)?;
        if word >= threshold {
            return Ok(start + (word % interval));
        }
    }
}

/// Reads the next 4 bytes as a little-endian `u32`.
pub(crate) fn read_u32<S: RandomSource + ?Sized>(source: &mut S) -> Result<u32, Error> {
    let mut bytes = [0u8; 4];
    source.try_fill_bytes(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}

/// Reads the next 8 bytes as a little-endian `u64`.
pub(crate) fn read_u64<S: RandomSource + ?Sized>(source: &mut S) -> Result<u64, Error> {
    let mut bytes = [0u8; 8];
    source.try_fill_bytes(&mut bytes)?;
    Ok(u64::from_le_bytes(bytes))
}

/// Samples an unbiased value from an integer range.
///
/// Bounds are validated before any byte is consumed, and the interval is sampled with rejection
/// sampling. The sampled value only depends on the interval, never on the target word size.
pub(crate) fn range<R, B, S>(source: &mut S, bounds: B) -> Result<R, Error>
where
    R: RangeEndpoint,
    B: SampleRange<R>,
    S: RandomSource + ?Sized,
{
    let (start, end) = bounds.bounds()?;
    R::sample_bounded(source, start, end)
}

/// Samples a uniform `f32` in `[0, 1)` from the top 24 bits of one word.
pub(crate) fn unit_f32<S: RandomSource + ?Sized>(source: &mut S) -> Result<f32, Error> {
    Ok((read_u32(source)? >> 8) as f32 * (1.0 / 16_777_216.0))
}

/// Samples a uniform `f64` in `[0, 1)` from the top 53 bits of one word.
pub(crate) fn unit_f64<S: RandomSource + ?Sized>(source: &mut S) -> Result<f64, Error> {
    Ok((read_u64(source)? >> 11) as f64 * (1.0 / 9_007_199_254_740_992.0))
}

/// Decides the rational probability `numerator / denominator`.
///
/// A zero or full probability returns without consuming bytes. Otherwise the source samples an
/// integer in `[0, denominator)` and compares it with `numerator`.
pub(crate) fn chance<S: RandomSource + ?Sized>(source: &mut S, numerator: u64, denominator: u64) -> Result<bool, Error> {
    if denominator == 0 || numerator > denominator {
        return Err(Error::new(ErrorKind::InvalidProbability));
    }
    if numerator == 0 {
        return Ok(false);
    }
    if numerator == denominator {
        return Ok(true);
    }
    Ok(range(source, 0u64..denominator)? < numerator)
}

/// Samples a uniform index in `0..len`.
///
/// An empty collection returns `None` without consuming bytes. The index is always sampled as a
/// `u64` interval, so the same collection length yields the same index on every target.
pub(crate) fn index<S: RandomSource + ?Sized>(source: &mut S, len: usize) -> Result<Option<usize>, Error> {
    if len == 0 {
        return Ok(None);
    }
    let bound = u64::try_from(len).map_err(|_| Error::new(ErrorKind::LengthOverflow))?;
    let sampled = range(source, 0u64..bound)?;
    let index = usize::try_from(sampled).map_err(|_| Error::new(ErrorKind::LengthOverflow))?;
    Ok(Some(index))
}

/// Samples a reference to one element, or `None` for an empty slice.
pub(crate) fn choose<'a, T, S: RandomSource + ?Sized>(source: &mut S, items: &'a [T]) -> Result<Option<&'a T>, Error> {
    match index(source, items.len())? {
        Some(index) => Ok(Some(&items[index])),
        None => Ok(None),
    }
}

/// Shuffles `items` with descending Fisher-Yates.
///
/// Each step samples an index in `0..i + 1` through the `u64` index path. A slice of length 0 or 1
/// is left untouched without consuming bytes. If the source fails mid-way, the swaps already
/// performed are kept and the error is returned.
pub(crate) fn shuffle<T, S: RandomSource + ?Sized>(source: &mut S, items: &mut [T]) -> Result<(), Error> {
    for i in (1..items.len()).rev() {
        let Some(target) = index(source, i + 1)?
        else {
            return Err(Error::new(ErrorKind::InvalidRange));
        };
        items.swap(i, target);
    }
    Ok(())
}
