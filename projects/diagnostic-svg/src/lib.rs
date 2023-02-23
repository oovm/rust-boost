//! SVG diagnostic preview rendering for structured diagnostics.

#![deny(missing_docs)]

mod render;

pub use render::{structured_set_to_svg, structured_to_svg, RenderError};
