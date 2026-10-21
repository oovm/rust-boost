#![doc = include_str!("readme.md")]
#![deny(missing_docs)]

pub mod fmt;

#[cfg(feature = "console")]
mod render;
#[cfg(feature = "console")]
mod sink;
#[cfg(feature = "console")]
mod subscriber;

#[cfg(feature = "console")]
pub use render::{render, render_plain};
#[cfg(feature = "console")]
pub use sink::{StderrFallbackSink, StderrSubscriberSink};
#[cfg(feature = "console")]
pub use subscriber::install_global_subscriber;
