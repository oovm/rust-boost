#![doc = include_str!("../readme.md")]
#![deny(missing_docs)]
#![no_std]

mod chacha;
mod error;
mod range;
mod sampling;
mod seed;
mod source;
mod stream;

pub use error::{Error, ErrorKind};
pub use range::{RangeEndpoint, SampleRange};
pub use seed::Seed;
pub use source::RandomSource;
pub use stream::Stream;
