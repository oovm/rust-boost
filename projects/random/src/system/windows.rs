//! Windows entropy backend.
//!
//! `BCryptGenRandom` with `BCRYPT_USE_SYSTEM_PREFERRED_RNG` is documented from Windows Vista on,
//! and is bound directly here rather than through another random crate. The backend keeps the
//! `NTSTATUS` value, so a failure is reported with its own status domain instead of a generic
//! message.

use core::{cmp, ffi::c_void, ptr};

use super::SystemEntropy;
use crate::error::EntropyError;

/// Asks BCrypt for the system-preferred random number generator, so no algorithm handle is needed.
const USE_SYSTEM_PREFERRED_RNG: u32 = 0x0000_0002;

/// `STATUS_NOT_SUPPORTED`, reported when the platform cannot serve the request.
const STATUS_NOT_SUPPORTED: i32 = 0xC000_00BBu32 as i32;

#[link(name = "bcrypt")]
unsafe extern "system" {
    fn BCryptGenRandom(algorithm: *mut c_void, buffer: *mut u8, length: u32, flags: u32) -> i32;
}

/// Fills `dst` with host entropy, or fails without delivering partial output.
pub(super) fn fill(entropy: &mut SystemEntropy, dst: &mut [u8]) -> Result<(), EntropyError> {
    let mut written = 0;
    while written < dst.len() {
        // The requested length is a `ULONG`, so a longer buffer is split into pieces.
        let piece = cmp::min(dst.len() - written, u32::MAX as usize);
        let head = dst[written..].as_mut_ptr();
        let status = unsafe { BCryptGenRandom(ptr::null_mut(), head, piece as u32, USE_SYSTEM_PREFERRED_RNG) };
        match status {
            0 => written += piece,
            STATUS_NOT_SUPPORTED => {
                entropy.mark_unsupported();
                return Err(EntropyError::NtStatus(STATUS_NOT_SUPPORTED));
            }
            code => return Err(EntropyError::NtStatus(code)),
        }
    }
    Ok(())
}
