//! The sampling facade over one byte source.

use core::ops::RangeBounds;

use crate::{
    distribution::Uniform,
    error::{RandomError, SourceError},
    sampling,
    source::RandomSource,
};

/// Sampling facade that owns one byte source.
///
/// One set of inherent methods serves every source, so an algorithm and a secure handle no longer
/// each carry a copy of the sampling entries. The facade is not a mandatory middle layer either: a
/// distribution object accepts any [`RandomSource`] directly, and `Random` itself implements
/// [`RandomSource`], so a distribution can sample through it too.
///
/// Entries that only read bytes return [`SourceError`]. Entries with parameters or numerical work
/// return [`RandomError`], which preserves the original source failure unchanged.
pub struct Random<S> {
    source: S,
}

impl<S> Random<S> {
    /// Wraps a byte source.
    pub fn new(source: S) -> Self {
        Self { source }
    }

    /// Returns the wrapped source.
    pub fn into_inner(self) -> S {
        self.source
    }
}

impl<S: RandomSource> Random<S> {
    /// Fills `dst` with the next bytes of the source.
    pub fn fill(&mut self, dst: &mut [u8]) -> Result<(), SourceError> {
        self.source.try_fill_bytes(dst)
    }

    /// Reads the next 4 bytes as a little-endian `u32`.
    pub fn u32(&mut self) -> Result<u32, SourceError> {
        sampling::read_u32(self)
    }

    /// Reads the next 8 bytes as a little-endian `u64`.
    pub fn u64(&mut self) -> Result<u64, SourceError> {
        sampling::read_u64(self)
    }

    /// Samples a uniform `f32` in `[0, 1)` from the top 24 bits of one word.
    pub fn unit_f32(&mut self) -> Result<f32, SourceError> {
        sampling::unit_f32(self)
    }

    /// Samples a uniform `f64` in `[0, 1)` from the top 53 bits of one word.
    pub fn unit_f64(&mut self) -> Result<f64, SourceError> {
        sampling::unit_f64(self)
    }

    /// Samples an unbiased value from a `u32` range.
    ///
    /// The bounds are normalized and validated during construction, so an illegal range fails
    /// before any byte is consumed.
    pub fn range_u32<B: RangeBounds<u32>>(&mut self, bounds: B) -> Result<u32, RandomError> {
        Uniform::<u32>::new(bounds)?.sample(self)
    }

    /// Samples an unbiased value from a `u64` range.
    pub fn range_u64<B: RangeBounds<u64>>(&mut self, bounds: B) -> Result<u64, RandomError> {
        Uniform::<u64>::new(bounds)?.sample(self)
    }

    /// Samples an unbiased value from an `i32` range.
    pub fn range_i32<B: RangeBounds<i32>>(&mut self, bounds: B) -> Result<i32, RandomError> {
        Uniform::<i32>::new(bounds)?.sample(self)
    }

    /// Samples an unbiased value from an `i64` range.
    pub fn range_i64<B: RangeBounds<i64>>(&mut self, bounds: B) -> Result<i64, RandomError> {
        Uniform::<i64>::new(bounds)?.sample(self)
    }

    /// Decides the rational probability `numerator / denominator`.
    ///
    /// A zero or full probability returns without consuming bytes.
    pub fn chance(&mut self, numerator: u64, denominator: u64) -> Result<bool, RandomError> {
        sampling::chance(self, numerator, denominator)
    }

    /// Samples a uniform index in `0..len`.
    ///
    /// An empty collection returns `None` without consuming bytes.
    pub fn index(&mut self, len: usize) -> Result<Option<usize>, RandomError> {
        sampling::index(self, len)
    }

    /// Samples a reference to one element, or `None` for an empty slice.
    pub fn choose<'a, T>(&mut self, items: &'a [T]) -> Result<Option<&'a T>, RandomError> {
        sampling::choose(self, items)
    }

    /// Shuffles `items` with descending Fisher-Yates.
    ///
    /// A slice of length 0 or 1 is left untouched without consuming bytes. A failure mid-way means a
    /// possibly partial rearrangement, not a rollback.
    pub fn shuffle<T>(&mut self, items: &mut [T]) -> Result<(), RandomError> {
        sampling::shuffle(self, items)
    }
}

impl<S: RandomSource> RandomSource for Random<S> {
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), SourceError> {
        self.fill(dst)
    }
}
