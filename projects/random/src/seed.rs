use core::fmt::{self, Debug, Formatter};

use crate::{entropy::EntropySource, error::RandomError, secure::Secure};

/// 32-byte key of the deterministic stream profile.
///
/// A seed carries no entropy guarantee of its own. `Seed::from_u64` only spans 64 bits, so it must
/// not be used for secrets.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Seed([u8; 32]);

impl Seed {
    /// Creates a seed from its exact 32-byte encoding.
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Creates a seed from a 64-bit value.
    ///
    /// The first 8 bytes are the little-endian encoding of `value`, and the remaining 24 bytes are
    /// zero. This does not expand the entropy of `value`.
    pub const fn from_u64(value: u64) -> Self {
        let mut bytes = [0u8; 32];
        let encoded = value.to_le_bytes();
        let mut index = 0;
        while index < 8 {
            bytes[index] = encoded[index];
            index += 1;
        }
        Self(bytes)
    }

    /// Generates a seed from a host entropy source.
    ///
    /// Only the passed [`Secure`] handle is read. Nothing opens a system source implicitly, and a
    /// failure is returned instead of a deterministic fallback seed.
    pub fn generate<E: EntropySource>(secure: &mut Secure<E>) -> Result<Self, RandomError> {
        let mut bytes = [0u8; 32];
        secure.fill(&mut bytes)?;
        Ok(Self(bytes))
    }

    /// Returns the 32-byte encoding for caller-owned replay records.
    pub const fn to_bytes(self) -> [u8; 32] {
        self.0
    }
}

impl Debug for Seed {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("Seed(<redacted>)")
    }
}
