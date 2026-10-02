#![doc = include_str!("../readme.md")]
#![deny(missing_docs)]

mod access;
mod identity;
mod range;
mod snapshot;
mod text;

#[cfg(feature = "std")]
mod memory;
mod wire;

pub use access::{AccessError, ByteAccess, PartialByteAccess};
pub use identity::{IdentityError, SourceIdentity, SourceLabel, SourceRef, SourceRevision, SourceSpan};
pub use range::{ByteRange, RangeError};
pub use snapshot::{SnapshotHandle, SnapshotRef};
pub use text::{LineIndex, TextCoords, TextView};

#[cfg(feature = "std")]
pub use memory::{MemorySnapshot, MemoryStore};
pub use wire::{decode_source_ref, encode_source_ref, WireError};
