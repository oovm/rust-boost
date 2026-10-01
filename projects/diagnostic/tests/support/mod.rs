pub mod fixtures;

#[cfg(feature = "serde")]
pub mod on_disk;

#[cfg(feature = "serde")]
pub mod wire;

#[cfg(feature = "terminal")]
pub mod structured_render;
