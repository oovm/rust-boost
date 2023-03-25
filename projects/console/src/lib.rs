#![doc = include_str!("../readme.md")]
#![deny(missing_docs)]

#[macro_use]
mod macros;

mod color;
mod draw;
mod paint;
mod sink;
mod style;
mod subscriber;

pub use color::Color;
pub use draw::{Background, Console, Foreground, Palette, StreamAwareFmt, StreamType};
pub use paint::Paint;
pub use sink::StderrFallbackSink;
pub use style::Style;
pub use subscriber::install_global_subscriber;
