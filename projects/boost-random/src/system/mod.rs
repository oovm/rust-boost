//! Platform entropy backends owned by this crate.
//!
//! The crate owns backend selection, capability facts, chunking, short-read handling, and error
//! classification. System SDKs and ABI bindings are used only as platform bindings: no random
//! source crate is linked, and a target without an implemented and tested backend reports
//! [`EntropyError::Unsupported`](crate::EntropyError::Unsupported) instead of guessing.

use crate::{entropy::EntropySource, error::EntropyError};

#[cfg(all(feature = "system", target_vendor = "apple"))]
mod apple;
#[cfg(all(feature = "system", target_os = "linux"))]
mod linux;
#[cfg(all(feature = "system", target_os = "wasi"))]
mod wasi;
#[cfg(all(feature = "web", target_arch = "wasm32", target_os = "unknown"))]
mod web;
#[cfg(all(feature = "system", windows))]
mod windows;

#[cfg(all(feature = "system", target_vendor = "apple"))]
use apple::fill as host_fill;
#[cfg(all(feature = "system", target_os = "linux"))]
use linux::fill as host_fill;
#[cfg(all(feature = "system", target_os = "wasi"))]
use wasi::fill as host_fill;
#[cfg(all(feature = "web", target_arch = "wasm32", target_os = "unknown"))]
use web::fill as host_fill;
#[cfg(all(feature = "system", windows))]
use windows::fill as host_fill;

/// The capability fact cached by a handle.
///
/// Only a permanent platform fact is cached, because availability must not be remembered: one
/// successful request does not prove that the next one succeeds, and an uninitialized pool or a
/// changed permission must never be stored as a permanent failure. A request therefore always goes
/// to the host while nothing permanent is known, and only a backend that proved the request
/// permanently unsupported stops later requests.
enum Capability {
    /// Nothing permanent is known yet, so requests go to the host.
    Unknown,
    /// The platform proved the request permanently unsupported, so later requests fail directly.
    Unsupported,
}

/// Handle to the entropy backends owned by this crate.
///
/// A handle owns only the capability fact of the compiled backend and never caches random bytes.
/// It therefore needs no global registry or singleton and can be created per use.
pub struct SystemEntropy {
    capability: Capability,
}

impl SystemEntropy {
    pub(crate) fn new() -> Self {
        Self { capability: Capability::Unknown }
    }

    /// Returns the cached capability fact, which is only permanent when the host proved the request
    /// unsupported for good.
    fn unsupported(&self) -> bool {
        matches!(self.capability, Capability::Unsupported)
    }

    /// Records that the host reported a permanently unsupported request.
    fn mark_unsupported(&mut self) {
        self.capability = Capability::Unsupported;
    }
}

impl EntropySource for SystemEntropy {
    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), EntropyError> {
        if self.unsupported() {
            return Err(EntropyError::Unsupported);
        }
        dispatch(self, dst)
    }
}

/// Requests entropy from the compiled platform backend.
#[cfg(any(
    all(feature = "system", windows),
    all(feature = "system", target_os = "linux"),
    all(feature = "system", target_os = "wasi"),
    all(feature = "system", target_vendor = "apple"),
    all(feature = "web", target_arch = "wasm32", target_os = "unknown"),
))]
fn dispatch(entropy: &mut SystemEntropy, dst: &mut [u8]) -> Result<(), EntropyError> {
    host_fill(entropy, dst)
}

/// Reports that this target has no implemented and tested backend.
#[cfg(not(any(
    all(feature = "system", windows),
    all(feature = "system", target_os = "linux"),
    all(feature = "system", target_os = "wasi"),
    all(feature = "system", target_vendor = "apple"),
    all(feature = "web", target_arch = "wasm32", target_os = "unknown"),
)))]
fn dispatch(entropy: &mut SystemEntropy, _dst: &mut [u8]) -> Result<(), EntropyError> {
    entropy.mark_unsupported();
    Err(EntropyError::Unsupported)
}
