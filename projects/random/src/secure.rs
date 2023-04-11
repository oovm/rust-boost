use crate::{
    entropy::EntropySource,
    error::RandomError,
    range::{RangeEndpoint, SampleRange},
    sampling,
    source::RandomSource,
    system::SystemEntropy,
};

/// Handle to a host entropy source.
///
/// `Secure` owns no random state. It has no seed, no seek, no cloneable state, and no
/// deterministic fallback, so a failure is reported instead of being replaced by weaker bytes.
///
/// The sampling entries below are shared with [`Stream`](crate::Stream): both types delegate to the
/// same functions, so one mapping and one consumption order serve both. The difference stays in the
/// bytes: `Secure` reads host entropy, so the same call sequence does not produce the same values,
/// and every non-empty request can fail.
pub struct Secure<E = SystemEntropy> {
    provider: E,
}

impl Secure<SystemEntropy> {
    /// Creates a handle to the platform entropy backends owned by this crate.
    ///
    /// Construction neither probes nor promises the platform: the first non-empty request does.
    /// A target without a compiled backend reports
    /// [`ErrorKind::Unsupported`](crate::RandomErrorKind::Unsupported) for every non-empty request.
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
    pub fn fill(&mut self, dst: &mut [u8]) -> Result<(), RandomError> {
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

    /// Reads the next 4 bytes of host entropy as a little-endian `u32`.
    pub fn u32(&mut self) -> Result<u32, RandomError> {
        sampling::read_u32(self)
    }

    /// Reads the next 8 bytes of host entropy as a little-endian `u64`.
    pub fn u64(&mut self) -> Result<u64, RandomError> {
        sampling::read_u64(self)
    }

    /// Samples an unbiased value from an integer range over host entropy.
    ///
    /// Bounds are validated before any byte is requested, and the interval is sampled with the same
    /// rejection mapping as [`Stream::range`](crate::Stream::range). A single-value interval
    /// requests no entropy.
    pub fn range<R, B>(&mut self, bounds: B) -> Result<R, RandomError>
    where
        R: RangeEndpoint,
        B: SampleRange<R>,
    {
        sampling::range(self, bounds)
    }

    /// Samples a uniform `f32` in `[0, 1)` from the top 24 bits of one word.
    pub fn unit_f32(&mut self) -> Result<f32, RandomError> {
        sampling::unit_f32(self)
    }

    /// Samples a uniform `f64` in `[0, 1)` from the top 53 bits of one word.
    pub fn unit_f64(&mut self) -> Result<f64, RandomError> {
        sampling::unit_f64(self)
    }

    /// Decides the rational probability `numerator / denominator` from host entropy.
    ///
    /// A zero or full probability returns without requesting entropy, and an out-of-range rational
    /// probability fails before any byte is requested.
    pub fn chance(&mut self, numerator: u64, denominator: u64) -> Result<bool, RandomError> {
        sampling::chance(self, numerator, denominator)
    }

    /// Samples a uniform index in `0..len`.
    ///
    /// An empty collection returns `None` without requesting entropy, and the index is always
    /// sampled as a `u64` interval so the same collection length uses the same mapping on every
    /// target.
    pub fn index(&mut self, len: usize) -> Result<Option<usize>, RandomError> {
        sampling::index(self, len)
    }

    /// Samples a reference to one element, or `None` for an empty slice.
    pub fn choose<'a, T>(&mut self, items: &'a [T]) -> Result<Option<&'a T>, RandomError> {
        sampling::choose(self, items)
    }

    /// Shuffles `items` with descending Fisher-Yates over host entropy.
    ///
    /// A slice of length 0 or 1 is left untouched without requesting entropy. If the provider fails
    /// mid-way, the swaps already performed are kept and the failure is returned, so a caller must
    /// treat a failure as a possibly partial rearrangement.
    pub fn shuffle<T>(&mut self, items: &mut [T]) -> Result<(), RandomError> {
        sampling::shuffle(self, items)
    }
}

/// A handle to host entropy is also a random source, so the shared sampling kernel can be driven
/// by it.
///
/// The implementation delegates to [`Secure::fill`], which clears the whole target before
/// reporting a failure. This direction is the only conversion between the two interfaces: an
/// arbitrary [`RandomSource`] never becomes an [`EntropySource`], because identical method shapes
/// carry no security guarantee.
impl<E: EntropySource> RandomSource for Secure<E> {
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), RandomError> {
        self.fill(dst)
    }
}
