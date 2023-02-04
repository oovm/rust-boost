#![doc = include_str!("readme.md")]
#![deny(missing_docs)]

mod cache;
mod identifier;
mod provider;
mod text;

pub use crate::{
    cache::SourceCache,
    identifier::{SourceID, SourcePath},
    provider::{ProviderError, SourceProvider, SourceRef},
    text::{SourceLine, SourceSpan, SourceText},
};
pub use url::Url;
