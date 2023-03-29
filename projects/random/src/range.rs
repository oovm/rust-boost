use core::ops::{Range, RangeInclusive};

use crate::{
    error::{Error, ErrorKind},
    sampling,
    stream::Stream,
};

/// Sealed marker that prevents downstream implementations.
mod sealed {
    /// Marker implemented by the supported range types.
    pub trait Sealed {}
}

/// Integer endpoint of a range accepted by [`Stream::range`](crate::Stream::range).
///
/// The trait is sealed: only `u32`, `u64`, `i32`, and `i64` are endpoints. Sampling always uses the
/// unsigned width of the value, so a 32-bit and a 64-bit target consume the same bytes for the
/// same interval.
pub trait RangeEndpoint: sealed::Sealed + Copy {
    /// Returns the value one step below `self`, or `None` when `self` is already the minimum.
    #[doc(hidden)]
    fn predecessor(self) -> Option<Self>;

    /// Samples one value from the inclusive bounds `start..=end`.
    #[doc(hidden)]
    fn sample_bounded(stream: &mut Stream, start: Self, end: Self) -> Result<Self, Error>;
}

/// Integer range accepted by [`Stream::range`](crate::Stream::range).
///
/// The trait is sealed and implemented for `Range` and `RangeInclusive` only.
pub trait SampleRange<R: RangeEndpoint>: sealed::Sealed {
    /// Returns the inclusive bounds of this range.
    ///
    /// A `Range` that contains no representable value, such as `0..0`, fails here.
    fn bounds(self) -> Result<(R, R), Error>;
}

impl sealed::Sealed for u32 {}
impl sealed::Sealed for u64 {}
impl sealed::Sealed for i32 {}
impl sealed::Sealed for i64 {}
impl<R: sealed::Sealed> sealed::Sealed for Range<R> {}
impl<R: sealed::Sealed> sealed::Sealed for RangeInclusive<R> {}

impl RangeEndpoint for u32 {
    fn predecessor(self) -> Option<Self> {
        self.checked_sub(1)
    }

    fn sample_bounded(stream: &mut Stream, start: Self, end: Self) -> Result<Self, Error> {
        sampling::sample_inclusive(stream, start, end)
    }
}

impl RangeEndpoint for u64 {
    fn predecessor(self) -> Option<Self> {
        self.checked_sub(1)
    }

    fn sample_bounded(stream: &mut Stream, start: Self, end: Self) -> Result<Self, Error> {
        sampling::sample_inclusive(stream, start, end)
    }
}

impl RangeEndpoint for i32 {
    fn predecessor(self) -> Option<Self> {
        self.checked_sub(1)
    }

    fn sample_bounded(stream: &mut Stream, start: Self, end: Self) -> Result<Self, Error> {
        sampling::sample_inclusive(stream, order_i32(start), order_i32(end)).map(unorder_i32)
    }
}

impl RangeEndpoint for i64 {
    fn predecessor(self) -> Option<Self> {
        self.checked_sub(1)
    }

    fn sample_bounded(stream: &mut Stream, start: Self, end: Self) -> Result<Self, Error> {
        sampling::sample_inclusive(stream, order_i64(start), order_i64(end)).map(unorder_i64)
    }
}

impl<R: RangeEndpoint> SampleRange<R> for Range<R> {
    fn bounds(self) -> Result<(R, R), Error> {
        match self.end.predecessor() {
            Some(end) => Ok((self.start, end)),
            None => Err(Error::new(ErrorKind::InvalidRange)),
        }
    }
}

impl<R: RangeEndpoint> SampleRange<R> for RangeInclusive<R> {
    fn bounds(self) -> Result<(R, R), Error> {
        Ok(self.into_inner())
    }
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
