use crate::{entropy::EntropySource, error::Error, system::SystemEntropy};

/// Handle to a host entropy source.
///
/// `Secure` owns no random state. It has no seed, no seek, no cloneable state, and no
/// deterministic fallback, so a failure is reported instead of being replaced by weaker bytes.
pub struct Secure<E = SystemEntropy> {
    provider: E,
}

impl Secure<SystemEntropy> {
    /// Creates a handle to the platform entropy backends owned by this crate.
    ///
    /// Construction neither probes nor promises the platform: the first non-empty request does.
    /// A target without a compiled backend reports
    /// [`ErrorKind::Unsupported`](crate::ErrorKind::Unsupported) for every non-empty request.
    pub fn system() -> Self {
        Self { provider: SystemEntropy::new() }
    }
}

impl<E: EntropySource> Secure<E> {
    /// Creates a handle from an explicit host entropy provider.
    pub fn with_entropy(provider: E) -> Self {
        Self { provider }
    }

    /// Fills `dst` with host entropy.
    ///
    /// An empty request succeeds without touching the host. When the provider fails, the entire
    /// `dst` is cleared before the failure is returned, so a caller never receives unmarked partial
    /// entropy as success.
    pub fn fill(&mut self, dst: &mut [u8]) -> Result<(), Error> {
        if dst.is_empty() {
            return Ok(());
        }
        match self.provider.try_fill_bytes(dst) {
            Ok(()) => Ok(()),
            Err(error) => {
                dst.fill(0);
                Err(error)
            }
        }
    }
}
