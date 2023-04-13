//! Web entropy backend for `wasm32-unknown-unknown`.
//!
//! The target triple cannot express whether the module runs in a browser window, in a worker, or in
//! Node, so the backend asks the host at run time instead of guessing from the target: the `crypto`
//! property of the global object is tested with `Reflect.get`, and a host without it reports a
//! permanent unsupported platform.
//!
//! `getRandomValues` rejects a request longer than 65,536 bytes, so a longer buffer is split. The
//! bridge is a few lines of `wasm-bindgen` glue compiled only for this feature and target: a native
//! build never sees it, because the module and the dependency are both gated.

use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

use super::SystemEntropy;
use crate::error::{EntropyError, WebError};

/// Largest buffer `getRandomValues` accepts in one call.
const REQUEST_LIMIT: usize = 65_536;

#[wasm_bindgen]
extern "C" {
    /// The global object, which holds `crypto` in every supported host.
    #[wasm_bindgen(thread_local_v2, js_name = globalThis)]
    static GLOBAL: JsValue;

    /// Returns the value of a property, or `undefined` when the property is absent.
    #[wasm_bindgen(js_namespace = ["Reflect"], js_name = get, catch)]
    fn reflect_get(target: &JsValue, key: &str) -> Result<JsValue, JsValue>;

    /// Fills `buffer` from the host cryptographic source.
    ///
    /// The host throws when the buffer exceeds its own limit, and the call is bound with `catch` so
    /// the thrown value becomes an error instead of unwinding into the module.
    #[wasm_bindgen(js_namespace = ["crypto"], js_name = getRandomValues, catch)]
    fn get_random_values(buffer: &mut [u8]) -> Result<(), JsValue>;
}

/// Returns the host `crypto` object, or `None` when the host has none or it cannot be probed.
fn host_crypto() -> Option<JsValue> {
    GLOBAL.with(|global| match reflect_get(global, "crypto") {
        Ok(value) if !value.is_undefined() && !value.is_null() => Some(value.clone()),
        _ => None,
    })
}

/// Fills `dst` with host entropy, or fails without delivering partial output.
pub(super) fn fill(entropy: &mut SystemEntropy, dst: &mut [u8]) -> Result<(), EntropyError> {
    if host_crypto().is_none() {
        // A missing `crypto` property is a property of the host, not of one request, so the fact is
        // recorded and later requests fail through it.
        entropy.mark_unsupported();
        return Err(EntropyError::Web(WebError::Probe));
    }

    let mut written = 0;
    while written < dst.len() {
        let piece = core::cmp::min(dst.len() - written, REQUEST_LIMIT);
        let chunk = &mut dst[written..written + piece];
        match get_random_values(chunk) {
            Ok(()) => written += piece,
            // The host refused the request. The thrown value is a JavaScript error object whose shape
            // is not part of this crate's contract, so no platform code is invented for it.
            Err(_) => return Err(EntropyError::Web(WebError::Rejected)),
        }
    }
    Ok(())
}
