#![doc = include_str!("../readme.md")]
#![deny(missing_docs)]
#![no_std]

mod chacha;
pub mod distribution;
mod entropy;
mod error;
mod math;
mod range;
mod sampling;
mod secure;
mod seed;
mod source;
mod stream;
mod system;

pub use entropy::EntropySource;
pub use error::{RandomError, RandomErrorKind};
pub use range::{RangeEndpoint, SampleRange};
pub use secure::Secure;
pub use seed::Seed;
pub use source::RandomSource;
pub use stream::Stream;
pub use system::SystemEntropy;
