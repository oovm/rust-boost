use crate::error::EntropyError;

/// Contract of a host entropy source.
///
/// An implementation must fill the whole slice or return an error, and it must not deliver unmarked
/// partial output as success. The implementation carries the security quality guarantee itself: a
/// fixed, predictable, or time-based source is not an [`EntropySource`], and this crate never
/// converts a [`RandomSource`](crate::RandomSource) into one, because identical method shapes say
/// nothing about security quality.
pub trait EntropySource {
    /// Fills `dst` with host entropy, or fails.
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), EntropyError>;
}

/// A borrowed source is usable wherever an owned source is expected.
impl<T: EntropySource + ?Sized> EntropySource for &mut T {
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), EntropyError> {
        (**self).try_fill_bytes(dst)
    }
}
