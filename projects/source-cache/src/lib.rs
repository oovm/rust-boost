#![doc = include_str!("readme.md")]
#![deny(missing_docs)]

pub mod provenance;

#[cfg(feature = "std")]
mod cache;
#[cfg(feature = "std")]
mod identifier;
#[cfg(feature = "std")]
mod provider;
#[cfg(feature = "std")]
mod text;

#[cfg(feature = "std")]
pub use crate::{
    cache::SourceCache,
    identifier::{SourceID, SourcePath},
    provider::{ProviderError, SourceProvider},
    text::{SourceLine, SourceSpan, SourceText},
};
#[cfg(feature = "std")]
pub use url::Url;

pub use provenance::{
    AddressSpaceRef, ByteRange, DocumentRef, MappingPrecision, MemberPath, MemberSegment, ObjectRef,
    RangeError, SemanticPath, SourceRef,
};
