//! Shared unbiased integer sampling of the `boost-chacha12-v1` profile.

use core::ops::{Add, Rem, Sub};

use crate::{
    error::{Error, ErrorKind},
    stream::Stream,
};

/// Unsigned word width consumed by the profile for integer sampling.
pub(crate) trait Word: Copy + Ord + Add<Output = Self> + Sub<Output = Self> + Rem<Output = Self> {
    /// Zero of this width.
    const ZERO: Self;

    /// One of this width.
    const ONE: Self;

    /// Consumes one word of this width from the stream.
    fn next_word(stream: &mut Stream) -> Result<Self, Error>;

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

    fn next_word(stream: &mut Stream) -> Result<Self, Error> {
        stream.u32()
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

    fn next_word(stream: &mut Stream) -> Result<Self, Error> {
        stream.u64()
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
pub(crate) fn sample_inclusive<W: Word>(stream: &mut Stream, start: W, end: W) -> Result<W, Error> {
    if end < start {
        return Err(Error::new(ErrorKind::InvalidRange));
    }
    if start == end {
        return Ok(start);
    }

    let span = end - start;
    let interval = span.wrapping_add(W::ONE);
    if interval == W::ZERO {
        return W::next_word(stream);
    }

    let threshold = interval.wrapping_neg() % interval;
    loop {
        let word = W::next_word(stream)?;
        if word >= threshold {
            return Ok(start + (word % interval));
        }
    }
}
