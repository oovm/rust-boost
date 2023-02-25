#![doc = include_str!("../readme.md")]
#![deny(missing_docs)]

mod platform;

/// Enables virtual terminal escape sequences on Windows consoles when possible.
///
/// Returns `true` when escape sequence support was successfully enabled and
/// `false` otherwise. On non-Windows targets this always returns `true`.
pub fn enable_virtual_terminal() -> bool {
    platform::enable_virtual_terminal()
}
