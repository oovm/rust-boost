//! WASI entropy backend.
//!
//! The backend binds the preview 1 `random_get` import directly and owns the errno mapping, so this
//! crate decides which failures are transient, permanent, or unsupported instead of delegating that
//! decision to another random crate. The host contract of `random_get` is to fill the whole buffer
//! before returning success, so no chunking or short-read loop is needed.

use super::SystemEntropy;
use crate::error::EntropyError;

// Preview 1 import of `random_get`, which fills `buf` with `buf_len` random bytes and returns a
// WASI `errno`. A foreign block does not accept a doc comment, so this is an ordinary comment.
#[link(wasm_import_module = "wasi_snapshot_preview1")]
unsafe extern "C" {
    fn random_get(buf: *mut u8, buf_len: usize) -> u16;
}

/// WASI `errno::intr`, which means the request was interrupted and may be retried.
const ERRNO_INTR: u16 = 27;

/// WASI `errno::nosys`, which means the host has no such import at all.
///
/// That is a permanent platform fact, so the handle records it instead of reporting a transient
/// failure, and later requests report the cached fact directly. The numbering follows the WASI
/// preview 1 `errno` enum.
const ERRNO_NOSYS: u16 = 52;

/// Fills `dst` with host entropy, or fails without delivering partial output.
pub(super) fn fill(entropy: &mut SystemEntropy, dst: &mut [u8]) -> Result<(), EntropyError> {
    loop {
        let code = unsafe { random_get(dst.as_mut_ptr(), dst.len()) };
        match code {
            0 => return Ok(()),
            // The request was interrupted before producing anything, so the same request is retried.
            ERRNO_INTR => continue,
            ERRNO_NOSYS => {
                entropy.mark_unsupported();
                return Err(EntropyError::Wasi(ERRNO_NOSYS));
            }
            // Any other errno is a host decision about this request, and the raw code is preserved in
            // its own WASI domain so the caller can tell which failure it was.
            code => return Err(EntropyError::Wasi(code)),
        }
    }
}
