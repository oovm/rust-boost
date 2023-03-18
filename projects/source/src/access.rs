use core::fmt::{self, Display, Formatter};

use crate::range::ByteRange;

/// Byte-oriented read access to an immutable snapshot.
pub trait ByteAccess {
    /// Total known byte length.
    fn len(&self) -> u64;

    /// Read a validated byte range.
    fn read(&self, range: ByteRange) -> Result<&[u8], AccessError>;

    /// Read `len` bytes starting at `offset`.
    fn read_at(&self, offset: u64, len: u64) -> Result<&[u8], AccessError> {
        let end = offset.checked_add(len).ok_or(AccessError::OutOfRange {
            len: self.len(),
            range: ByteRange::new(offset, offset).expect("zero-length range is valid"),
        })?;
        self.read(ByteRange::new(offset, end).expect("checked addition preserves ordering"))
    }
}

/// Byte access failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccessError {
    /// Requested range exceeds the known snapshot length.
    OutOfRange {
        /// Known snapshot length.
        len: u64,
        /// Requested range.
        range: ByteRange,
    },
    /// Host exposes only part of the requested range.
    NeedRange {
        /// Currently available bytes.
        available: ByteRange,
        /// Requested range.
        requested: ByteRange,
    },
    /// Bytes are not valid UTF-8 at the requested view.
    NotUtf8 {
        /// Offset of the invalid sequence.
        offset: u64,
    },
    /// Local snapshot handle no longer resolves.
    SnapshotReleased,
}

impl Display for AccessError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfRange { len, range } => {
                write!(f, "byte range [{}, {}) exceeds snapshot length {}", range.start(), range.end(), len)
            }
            Self::NeedRange { available, requested } => {
                write!(
                    f,
                    "requested byte range [{}, {}) but only [{}, {}) is available",
                    requested.start(),
                    requested.end(),
                    available.start(),
                    available.end()
                )
            }
            Self::NotUtf8 { offset } => write!(f, "invalid UTF-8 at byte offset {}", offset),
            Self::SnapshotReleased => f.write_str("snapshot handle has been released"),
        }
    }
}

/// Partial byte access used by hosts that expose only a known prefix.
pub struct PartialByteAccess<'a> {
    bytes: &'a [u8],
    available_end: u64,
}

impl<'a> PartialByteAccess<'a> {
    /// Create a partial view over `bytes` with `available_end` bytes readable.
    pub fn new(bytes: &'a [u8], available_end: u64) -> Result<Self, AccessError> {
        let len = u64::try_from(bytes.len()).map_err(|_| AccessError::OutOfRange {
            len: 0,
            range: ByteRange::new(0, 1).expect("constant range"),
        })?;
        if available_end > len {
            return Err(AccessError::OutOfRange {
                len,
                range: ByteRange::new(available_end, available_end).expect("zero-length range is valid"),
            });
        }
        Ok(Self { bytes, available_end })
    }
}

impl ByteAccess for PartialByteAccess<'_> {
    fn len(&self) -> u64 {
        self.available_end
    }

    fn read(&self, range: ByteRange) -> Result<&[u8], AccessError> {
        if range.end() > self.available_end {
            return Err(AccessError::NeedRange {
                available: ByteRange::new(0, self.available_end).expect("available end is ordered"),
                requested: range,
            });
        }
        let (start, end) = range.to_usize().map_err(|_| AccessError::OutOfRange {
            len: self.available_end,
            range,
        })?;
        Ok(&self.bytes[start..end])
    }
}

impl ByteAccess for [u8] {
    fn len(&self) -> u64 {
        self.len() as u64
    }

    fn read(&self, range: ByteRange) -> Result<&[u8], AccessError> {
        let len = self.len() as u64;
        if range.end() > len {
            return Err(AccessError::OutOfRange { len, range });
        }
        let (start, end) = range.to_usize().map_err(|_| AccessError::OutOfRange { len, range })?;
        Ok(&self[start..end])
    }
}
