//! Shared sampling entries of the profile.
//!
//! Every entry is written against [`RandomSource`], not against one concrete source type, so a
//! deterministic algorithm and a host entropy handle consume bytes through the same mapping,
//! rejection loop, and consumption order. Bytes are read as whole little-endian words of the
//! unsigned width of the endpoint, so the same interval consumes the same bytes on a 32-bit and a
//! 64-bit target.
//!
//! The layer is deliberately split by failure source: reading bytes and scalars can only fail at the
//! source, while entries with parameters or numerical work return [`RandomError`].

use core::ops::{Add, Rem, Sub};

use crate::{
    error::{RandomError, SourceError},
    source::RandomSource,
};

/// Unsigned word width consumed by the profile for integer sampling.
pub(crate) trait Word: Copy + Ord + Add<Output = Self> + Sub<Output = Self> + Rem<Output = Self> {
    /// Zero of this width.
    const ZERO: Self;

    /// One of this width.
    const ONE: Self;

    /// Consumes one word of this width from an arbitrary source.
    fn next_word<S: RandomSource + ?Sized>(source: &mut S) -> Result<Self, SourceError>;

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

    fn next_word<S: RandomSource + ?Sized>(source: &mut S) -> Result<Self, SourceError> {
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

    fn next_word<S: RandomSource + ?Sized>(source: &mut S) -> Result<Self, SourceError> {
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
/// The interval must be legal, that is `start <= end`. Every caller establishes that during
/// construction of its value object, so this entry consumes no byte before validating: an illegal
/// interval is never built, and an illegal request is never represented.
///
/// An interval of a single value returns that value without consuming bytes. A full-domain interval
/// accepts every word, so it consumes exactly one word. Otherwise the value is obtained by rejection
/// sampling: the profile discards words below `2^w mod n` and then reduces modulo `n`, which keeps
/// every value equally likely. A source failure propagates unchanged.
pub(crate) fn sample_inclusive<W: Word, S: RandomSource + ?Sized>(source: &mut S, start: W, end: W) -> Result<W, SourceError> {
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

/// Samples an unbiased `i32` from the inclusive interval, using the order-preserving transform.
pub(crate) fn sample_i32<S: RandomSource + ?Sized>(source: &mut S, start: i32, end: i32) -> Result<i32, SourceError> {
    sample_inclusive(source, order_i32(start), order_i32(end)).map(unorder_i32)
}

/// Samples an unbiased `i64` from the inclusive interval, using the order-preserving transform.
pub(crate) fn sample_i64<S: RandomSource + ?Sized>(source: &mut S, start: i64, end: i64) -> Result<i64, SourceError> {
    sample_inclusive(source, order_i64(start), order_i64(end)).map(unorder_i64)
}

/// Reads the next 4 bytes as a little-endian `u32`.
pub(crate) fn read_u32<S: RandomSource + ?Sized>(source: &mut S) -> Result<u32, SourceError> {
    let mut bytes = [0u8; 4];
    source.try_fill_bytes(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}

/// Reads the next 8 bytes as a little-endian `u64`.
pub(crate) fn read_u64<S: RandomSource + ?Sized>(source: &mut S) -> Result<u64, SourceError> {
    let mut bytes = [0u8; 8];
    source.try_fill_bytes(&mut bytes)?;
    Ok(u64::from_le_bytes(bytes))
}

/// Samples a uniform `f32` in `[0, 1)` from the top 24 bits of one word.
pub(crate) fn unit_f32<S: RandomSource + ?Sized>(source: &mut S) -> Result<f32, SourceError> {
    Ok((read_u32(source)? >> 8) as f32 * (1.0 / 16_777_216.0))
}

/// Samples a uniform `f64` in `[0, 1)` from the top 53 bits of one word.
pub(crate) fn unit_f64<S: RandomSource + ?Sized>(source: &mut S) -> Result<f64, SourceError> {
    Ok((read_u64(source)? >> 11) as f64 * (1.0 / 9_007_199_254_740_992.0))
}

/// Decides the rational probability `numerator / denominator`.
///
/// A zero or full probability returns without consuming bytes. An invalid rational probability fails
/// before any byte is read. Otherwise the source samples an integer in `[0, denominator)` and
/// compares it with `numerator`.
pub(crate) fn chance<S: RandomSource + ?Sized>(source: &mut S, numerator: u64, denominator: u64) -> Result<bool, RandomError> {
    if denominator == 0 || numerator > denominator {
        return Err(RandomError::InvalidProbability);
    }
    if numerator == 0 {
        return Ok(false);
    }
    if numerator == denominator {
        return Ok(true);
    }
    Ok(sample_inclusive(source, 0u64, denominator - 1)? < numerator)
}

/// Samples a uniform index in `0..len`.
///
/// An empty collection returns `None` without consuming bytes. The index is always sampled as a
/// `u64` interval, so the same collection length yields the same index on every target and a narrow
/// target never falls back to a narrower algorithm.
pub(crate) fn index<S: RandomSource + ?Sized>(source: &mut S, len: usize) -> Result<Option<usize>, RandomError> {
    if len == 0 {
        return Ok(None);
    }
    let bound = u64::try_from(len).map_err(|_| RandomError::LengthOverflow)?;
    let sampled = sample_inclusive(source, 0u64, bound - 1)?;
    let index = usize::try_from(sampled).map_err(|_| RandomError::LengthOverflow)?;
    Ok(Some(index))
}

/// Samples a reference to one element, or `None` for an empty slice.
pub(crate) fn choose<'a, T, S: RandomSource + ?Sized>(source: &mut S, items: &'a [T]) -> Result<Option<&'a T>, RandomError> {
    match index(source, items.len())? {
        Some(index) => Ok(Some(&items[index])),
        None => Ok(None),
    }
}

/// Shuffles `items` with descending Fisher-Yates.
///
/// Each step samples an index in `0..i + 1` through the `u64` index path. A slice of length 0 or 1
/// is left untouched without consuming bytes. If the source fails mid-way, the swaps already
/// performed are kept and the error is returned, so a failure means a possibly partial
/// rearrangement.
pub(crate) fn shuffle<T, S: RandomSource + ?Sized>(source: &mut S, items: &mut [T]) -> Result<(), RandomError> {
    for i in (1..items.len()).rev() {
        let Some(target) = index(source, i + 1)?
        else {
            return Err(RandomError::InvalidRange);
        };
        items.swap(i, target);
    }
    Ok(())
}

/// Maps a signed value onto the unsigned domain while preserving order.
fn order_i32(value: i32) -> u32 {
    (value as u32) ^ 0x8000_0000
}

/// Reverses [`order_i32`].
fn unorder_i32(value: u32) -> i32 {
    (value ^ 0x8000_0000) as i32
}

/// Maps a signed value onto the unsigned domain while preserving order.
fn order_i64(value: i64) -> u64 {
    (value as u64) ^ 0x8000_0000_0000_0000
}

/// Reverses [`order_i64`].
fn unorder_i64(value: u64) -> i64 {
    (value ^ 0x8000_0000_0000_0000) as i64
}
