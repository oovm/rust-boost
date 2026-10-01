fn accept(value: &str) -> bool {
    value.eq("1") || value.eq_ignore_ascii_case("true")
}

/// Enables ANSI terminal escape sequences on Windows consoles when
/// possible. Returns `true` if escape sequence support was successfully
/// enabled and `false` otherwise. On non-Windows targets, this method
/// always returns `true`.
///
/// Support for escape sequences in Windows consoles was added in the
/// Windows 10 anniversary update. For targets with older Windows
/// installations, this method is expected to return `false`.
///
/// The `DIAGNOSTIC_COLOR` environment variable must be set to `1` or
/// `true` before this function attempts to enable terminal support.
///
/// # Example
///
/// ```rust
/// use diagnostic::enable_ansi_color;
///
/// // A best-effort Windows ASCII terminal support enabling.
/// enable_ansi_color();
/// ```
#[inline]
pub fn enable_ansi_color() -> bool {
    match std::env::var("DIAGNOSTIC_COLOR") {
        Ok(value) if accept(&value) => terminal::enable_virtual_terminal(),
        _ => false,
    }
}
