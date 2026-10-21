//! Text styling, color, paint, and stream-aware formatting.

#[macro_use]
mod macros;

mod color;
mod draw;
mod paint;
mod style;

pub use color::Color;
pub use draw::{Background, Console, Foreground, Palette, StreamAwareFmt, StreamType};
pub use paint::Paint;
pub use style::Style;
