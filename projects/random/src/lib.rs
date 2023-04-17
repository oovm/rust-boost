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

pub use algorithms::ChaCha12;
pub use entropy::EntropySource;
pub use error::{EntropyError, RandomError, SourceError, WebError};
pub use random::Random;
pub use secure::Secure;
pub use source::RandomSource;
pub use system::SystemEntropy;
