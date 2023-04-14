//! Linux entropy backend.
//!
//! The backend calls the `getrandom` wrapper that the target's C library exposes. Minimum platform:
//! glibc 2.25 or musl 1.1.20, which provide the wrapper, on kernel 3.17 or later. The crate declares
//! the two symbols it calls itself, so chunking, short-read handling, and error classification stay
//! owned here instead of being borrowed from another crate's platform surface. The backend never
//! falls back to `/dev/urandom`, a timestamp, or a hardware instruction, so an unsupported platform
//! is reported instead of a weaker source being substituted.
//!
//! This module is selected for `target_os = "linux"` only. Android is a different target whose C
//! library exposes a different `errno` accessor, so it never compiles this module and keeps
//! reporting [`EntropyError::Unsupported`] until a separate backend is implemented and verified.

use core::{
    cmp,
    ffi::{c_int, c_uint, c_void},
};

use super::SystemEntropy;
use crate::error::EntropyError;

/// One request never asks for more than this many bytes, so the loop does not rely on a kernel
/// returning a large request in a single call.
const REQUEST_LIMIT: usize = 256;

/// The `EINTR` code of the target's error table.
///
/// No Linux architecture overrides this number, so one constant is correct for every
/// `target_os = "linux"` target instead of a value copied from the development machine.
const EINTR: c_int = 4;

/// The `EAGAIN` code of the target's error table. Like `EINTR`, no Linux architecture overrides it.
const EAGAIN: c_int = 11;

/// The `ENOSYS` code of the target's error table.
///
/// Unlike `EINTR` and `EAGAIN`, this number is not shared by every Linux architecture, because the
/// MIPS and SPARC families keep their own error tables. The constant is therefore stated per family
/// rather than assumed to hold for every Linux target.
#[cfg(any(
    target_arch = "mips",
    target_arch = "mips32r6",
    target_arch = "mips64",
    target_arch = "mips64r6"
))]
const ENOSYS: c_int = 89;

/// The `ENOSYS` code of the SPARC family, which keeps its own error table. See the MIPS variant.
#[cfg(any(target_arch = "sparc", target_arch = "sparc64"))]
const ENOSYS: c_int = 90;

/// The `ENOSYS` code shared by the remaining Linux architectures. See the MIPS variant.
#[cfg(not(any(
    target_arch = "mips",
    target_arch = "mips32r6",
    target_arch = "mips64",
    target_arch = "mips64r6",
    target_arch = "sparc",
    target_arch = "sparc64"
)))]
const ENOSYS: c_int = 38;

// The C-library symbols this backend calls, and nothing else.
unsafe extern "C" {
    /// `ssize_t getrandom(void *buf, size_t buflen, unsigned int flags)`.
    fn getrandom(buf: *mut c_void, buflen: usize, flags: c_uint) -> isize;

    /// `int *__errno_location(void)`, the thread-local `errno` accessor of glibc and musl.
    ///
    /// The accessor belongs to these two C libraries, so it is declared only for this target and is
    /// not treated as a universal Unix entry point.
    fn __errno_location() -> *mut c_int;
}

/// Fills `dst` with host entropy, or fails without delivering partial output.
pub(super) fn fill(entropy: &mut SystemEntropy, dst: &mut [u8]) -> Result<(), EntropyError> {
    let mut written = 0;
    while written < dst.len() {
        let piece = cmp::min(dst.len() - written, REQUEST_LIMIT);
        let head = dst[written..].as_mut_ptr().cast::<c_void>();
        match unsafe { getrandom(head, piece, 0) } {
            // A short read is expected, so the same request continues with the rest.
            returned if returned > 0 => written += returned as usize,
            // No progress and no errno were reported, so continuing would loop forever.
            0 => return Err(EntropyError::Failed),
            _ => match current_errno() {
                // The call was interrupted, so the same request is retried instead of failing.
                EINTR => continue,
                // The pool is not ready yet, which does not make the platform permanently unusable.
                EAGAIN => return Err(EntropyError::Unavailable),
                // The kernel has no such call, which is a permanent platform fact. The raw code is
                // kept, and later requests report the cached fact directly.
                ENOSYS => {
                    entropy.mark_unsupported();
                    return Err(EntropyError::Errno(ENOSYS));
                }
                code => return Err(EntropyError::Errno(code)),
            },
        }
    }
    Ok(())
}

/// Reads the thread-local `errno` reported by the target's C library.
fn current_errno() -> c_int {
    unsafe { *__errno_location() }
}
