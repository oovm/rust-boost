pub mod fixtures;

#[cfg(feature = "console")]
pub mod logger_projection;

#[cfg(feature = "console")]
pub mod console_emit;

#[cfg(feature = "serde")]
pub mod on_disk;

#[cfg(feature = "serde")]
pub mod wire;

#[cfg(feature = "terminal")]
pub mod structured_render;

#[cfg(feature = "terminal")]
pub mod terminal_golden;
