//! Path-keyed file access with overlay support and read consistency versions.

mod content;
mod disk;
mod error;
mod layered;
mod overlay;
mod version;

pub use content::FileContent;
pub use disk::DiskFs;
pub use error::VfsError;
pub use layered::LayeredFs;
pub use overlay::Overlay;
pub use version::FileVersion;
