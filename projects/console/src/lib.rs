#![doc = include_str!("../readme.md")]
#![deny(missing_docs)]

#[macro_use]
mod macros;

mod color;
mod draw;
mod paint;
mod render;
mod sink;
mod style;
mod subscriber;

pub use color::Color;
pub use draw::{Background, Console, Foreground, Palette, StreamAwareFmt, StreamType};
pub use paint::Paint;
pub use render::{render, render_plain};
pub use sink::{StderrFallbackSink, StderrSubscriberSink};
pub use style::Style;
pub use subscriber::install_global_subscriber;
