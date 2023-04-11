//! Linux entropy backend.
//!
//! The backend calls the `getrandom` system call through the `libc` platform binding. Minimum
//! platform: glibc 2.25 or musl 1.1.20, which expose the wrapper, on kernel 3.17 or later. The
//! backend never falls back to `/dev/urandom`, a timestamp, or a hardware instruction, so an
//! unsupported platform is reported instead of a weaker source being substituted.

use core::{cmp, ffi::c_void};

use super::SystemEntropy;
use crate::error::{RandomError, RandomErrorKind};

/// One request never asks for more than this many bytes, so the loop does not rely on a kernel
/// returning a large request in a single call.
const REQUEST_LIMIT: usize = 256;

/// Fills `dst` with host entropy, or fails without delivering partial output.
pub(super) fn fill(entropy: &mut SystemEntropy, dst: &mut [u8]) -> Result<(), RandomError> {
    let mut written = 0;
    while written < dst.len() {
        let piece = cmp::min(dst.len() - written, REQUEST_LIMIT);
        let head = dst[written..].as_mut_ptr().cast::<c_void>();
        match unsafe { libc::getrandom(head, piece, 0) } {
            // A short read is expected, so the same request continues with the rest.
            returned if returned > 0 => written += returned as usize,
            // No progress was made, so continuing would loop forever.
            0 => return Err(RandomError::new(RandomErrorKind::BackendFailure)),
            _ => match current_errno() {
                // The call was interrupted, so the same request is retried instead of failing.
                libc::EINTR => continue,
                // The pool is not ready yet, which does not make the platform permanently unusable.
                libc::EAGAIN => return Err(RandomError::with_platform_code(RandomErrorKind::EntropyUnavailable, libc::EAGAIN)),
                // The kernel has no such call, which is a permanent platform fact.
                libc::ENOSYS => {
                    entropy.mark_unsupported();
                    return Err(RandomError::with_platform_code(RandomErrorKind::Unsupported, libc::ENOSYS));
                }
                code => return Err(RandomError::with_platform_code(RandomErrorKind::BackendFailure, code)),
            },
        }
    }
    Ok(())
}

/// Reads the thread-local `errno` reported by the platform binding.
fn current_errno() -> i32 {
    unsafe { *libc::__errno_location() }
}
