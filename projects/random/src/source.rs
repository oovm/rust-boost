use crate::error::SourceError;

/// Minimal object-safe contract for a byte producer.
///
/// The interface only supplies bytes: it carries no seed, no algorithm identifier, no range, no
/// floating-point entry, and no sampling schedule, so a custom source does not have to implement a
/// stream cipher. An implementation must either fill the whole slice or return an error, and it must
/// not deliver unmarked partial output as success.
pub trait RandomSource {
    /// Fills `dst` with produced bytes, or fails.
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), SourceError>;
}
