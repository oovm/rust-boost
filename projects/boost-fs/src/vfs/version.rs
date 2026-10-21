use std::sync::atomic::{AtomicU64, Ordering};

/// Monotonic content version returned with file reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FileVersion(u64);

impl FileVersion {
    /// Builds a version token from on-disk metadata.
    pub fn from_disk(modified_nanos: u64, len: u64) -> Self {
        Self(modified_nanos ^ (len << 1))
    }

    /// Returns the raw version value.
    pub fn as_u64(self) -> u64 {
        self.0
    }

    pub(crate) fn next(seed: &AtomicU64) -> Self {
        Self(seed.fetch_add(1, Ordering::SeqCst))
    }
}
