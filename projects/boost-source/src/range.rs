use core::fmt::{self, Display, Formatter};

/// Half-open byte range `[start, end)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteRange {
    start: u64,
    end: u64,
}

impl ByteRange {
    /// Create a byte range and validate ordering.
    pub fn new(start: u64, end: u64) -> Result<Self, RangeError> {
        if start > end {
            return Err(RangeError { start, end });
        }
        Ok(Self { start, end })
    }

    /// Inclusive start offset.
    pub fn start(&self) -> u64 {
        self.start
    }

    /// Exclusive end offset.
    pub fn end(&self) -> u64 {
        self.end
    }

    /// Returns whether the range is empty.
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Byte length of the range.
    pub fn len(&self) -> u64 {
        self.end - self.start
    }

    /// Returns whether `offset` lies inside the half-open range.
    pub fn contains(&self, offset: u64) -> bool {
        self.start <= offset && offset < self.end
    }

    /// Convert the range to `usize` bounds when they fit.
    pub fn to_usize(&self) -> Result<(usize, usize), RangeError> {
        let start = usize::try_from(self.start).map_err(|_| RangeError { start: self.start, end: self.end })?;
        let end = usize::try_from(self.end).map_err(|_| RangeError { start: self.start, end: self.end })?;
        Ok((start, end))
    }

    /// Convert the range to LSP `u32` bounds when they fit.
    pub fn to_lsp_u32(&self) -> Result<(u32, u32), RangeError> {
        let start = u32::try_from(self.start).map_err(|_| RangeError { start: self.start, end: self.end })?;
        let end = u32::try_from(self.end).map_err(|_| RangeError { start: self.start, end: self.end })?;
        Ok((start, end))
    }
}

/// Invalid byte range ordering or overflow while narrowing coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RangeError {
    start: u64,
    end: u64,
}

impl Display for RangeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "invalid byte range [{}, {})", self.start, self.end)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for ByteRange {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ByteRange", 2)?;
        state.serialize_field("start", &self.start)?;
        state.serialize_field("end", &self.end)?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for ByteRange {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Raw {
            start: u64,
            end: u64,
        }
        let raw = Raw::deserialize(deserializer)?;
        Self::new(raw.start, raw.end).map_err(serde::de::Error::custom)
    }
}
