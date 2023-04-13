//! Apple entropy backend.
//!
//! The backend calls `getentropy` through the `libc` platform binding. Minimum platform: macOS
//! 10.12, iOS 10, tvOS 10, and watchOS 3. `getentropy` rejects a request larger than 256 bytes,
//! so the backend splits larger buffers and keeps the platform code of a failure.

use core::{cmp, ffi::c_void};

use super::SystemEntropy;
use crate::error::EntropyError;

/// The documented maximum request size of `getentropy`.
const REQUEST_LIMIT: usize = 256;

/// Fills `dst` with host entropy, or fails without delivering partial output.
pub(super) fn fill(_entropy: &mut SystemEntropy, dst: &mut [u8]) -> Result<(), EntropyError> {
    let mut written = 0;
    while written < dst.len() {
        let piece = cmp::min(dst.len() - written, REQUEST_LIMIT);
        let head = dst[written..].as_mut_ptr().cast::<c_void>();
        if unsafe { libc::getentropy(head, piece) } == 0 {
            written += piece;
            continue;
        }
        return Err(EntropyError::Errno(current_errno()));
    }
    Ok(())
}

/// Reads the thread-local `errno` reported by the platform binding.
fn current_errno() -> i32 {
    unsafe { *libc::__error() }
}
