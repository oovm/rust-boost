use crate::error::Error;

/// Minimal object-safe contract for a byte producer.
///
/// [`Stream`](crate::Stream) implements this interface. An implementation must either fill the
/// whole slice or return an error, and it must not deliver unmarked partial output as success.
pub trait RandomSource {
    /// Fills `dst` with produced bytes, or fails.
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Error>;
}
