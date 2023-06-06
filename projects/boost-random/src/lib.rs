#![doc = include_str!("readme.md")]
#![deny(missing_docs)]
#![no_std]

pub mod algorithms;
pub mod distribution;
mod entropy;
mod error;
mod math;
mod random;
mod sampling;
mod secure;
mod source;
mod system;

pub use crate::{
    algorithms::ChaCha12,
    entropy::EntropySource,
    error::{EntropyError, RandomError, SourceError, WebError},
    random::Random,
    secure::Secure,
    source::RandomSource,
    system::SystemEntropy,
};
