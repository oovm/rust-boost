//! Candidate names for temporary paths.

use std::io;

use random::Secure;

/// Number of entropy bytes used in a candidate name.
const SUFFIX_BYTES: usize = 16;

/// Builds a candidate name prefix for temporary paths.
///
/// The suffix comes from the host entropy source, so a candidate is not predictable from a timestamp,
/// a process id, or a counter. This only names a candidate: randomness does not prove uniqueness, so
/// every caller still creates the path atomically with `create_new` or `create_folder` and retries on
/// a collision. A host without a usable entropy source fails here instead of silently falling back to
/// a predictable name.
pub(crate) fn prefix(label: &str) -> io::Result<String> {
    let mut suffix = [0u8; SUFFIX_BYTES];
    let mut secure = Secure::system();
    secure.fill(&mut suffix).map_err(|error| io::Error::other(error.to_string()))?;
    Ok(format!(".fs-tools-{label}-{}", encode(&suffix)))
}

/// Encodes the entropy bytes as lowercase hexadecimal.
fn encode(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        text.push(char::from(DIGITS[usize::from(byte >> 4)]));
        text.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    text
}
