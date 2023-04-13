use crate::{entropy::EntropySource, error::SourceError, source::RandomSource, system::SystemEntropy};

/// Handle to a host entropy source.
///
/// `Secure` owns no random state. It has no seed, no seek, no cloneable state, and no deterministic
/// fallback, so a failure is reported instead of being replaced by weaker bytes.
///
/// The handle only supplies bytes. Sampling is done through [`Random`](crate::Random), which owns
/// the shared sampling entries, so this type cannot drift away from any other source.
pub struct Secure<E = SystemEntropy> {
    provider: E,
}

impl Secure<SystemEntropy> {
    /// Creates a handle to the platform entropy backends owned by this crate.
    ///
    /// Construction neither probes nor promises the platform: the first non-empty request does. A
    /// target without a compiled backend reports [`EntropyError::Unsupported`] for every non-empty
    /// request.
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
    /// entropy as success. The failure is wrapped as [`SourceError::Entropy`] at this boundary, so
    /// sampling code sees one source error type without losing the platform error inside it.
    pub fn fill(&mut self, dst: &mut [u8]) -> Result<(), SourceError> {
        if dst.is_empty() {
            return Ok(());
        }
        match self.provider.try_fill_bytes(dst) {
            Ok(()) => Ok(()),
            Err(error) => {
                dst.fill(0);
                Err(SourceError::Entropy(error))
            }
        }
    }
}

/// A handle to host entropy is also a random source, so the shared sampling entries can be driven by
/// it.
///
/// The implementation delegates to [`Secure::fill`], which clears the whole target before reporting
/// a failure. This direction is the only conversion between the two interfaces: an arbitrary
/// [`RandomSource`] never becomes an [`EntropySource`], because identical method shapes carry no
/// security guarantee.
impl<E: EntropySource> RandomSource for Secure<E> {
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), SourceError> {
        self.fill(dst)
    }
}
