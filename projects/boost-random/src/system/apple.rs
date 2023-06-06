//! Apple entropy backend.
//!
//! The backend calls `getentropy` from the system library. Minimum platform: macOS 10.12, iOS 10,
//! tvOS 10, and watchOS 3. `getentropy` rejects a request larger than 256 bytes, so the backend
//! splits larger buffers and keeps the platform code of a failure.
//!
//! The crate declares the two symbols it calls itself. Apple reports its thread-local `errno`
//! through `__error`, which is the accessor of this platform and deliberately not the glibc or musl
//! entry point used by the Linux backend. A failure only carries the raw code, so this backend needs
//! no error-table constants.

use core::{
    cmp,
    ffi::{c_int, c_void},
};

use super::SystemEntropy;
use crate::error::EntropyError;

/// The documented maximum request size of `getentropy`.
const REQUEST_LIMIT: usize = 256;

// The system-library symbols this backend calls, and nothing else.
unsafe extern "C" {
    /// `int getentropy(void *buf, size_t buflen)`.
    fn getentropy(buf: *mut c_void, buflen: usize) -> c_int;

    /// `int *__error(void)`, the thread-local `errno` accessor of the Apple system library.
    fn __error() -> *mut c_int;
}

/// Fills `dst` with host entropy, or fails without delivering partial output.
pub(super) fn fill(_entropy: &mut SystemEntropy, dst: &mut [u8]) -> Result<(), EntropyError> {
    let mut written = 0;
    while written < dst.len() {
        let piece = cmp::min(dst.len() - written, REQUEST_LIMIT);
        let head = dst[written..].as_mut_ptr().cast::<c_void>();
        if unsafe { getentropy(head, piece) } == 0 {
            written += piece;
            continue;
        }
        return Err(EntropyError::Errno(current_errno()));
    }
    Ok(())
}

/// Reads the thread-local `errno` reported by the system library.
fn current_errno() -> c_int {
    unsafe { *__error() }
}
