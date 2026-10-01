#[cfg(windows)]
mod windows_console {
    use std::os::raw::c_void;

    #[allow(non_camel_case_types)]
    type c_ulong = u32;
    #[allow(non_camel_case_types)]
    type c_int = i32;
    type DWORD = c_ulong;
    type LPDWORD = *mut DWORD;
    type HANDLE = *mut c_void;
    type BOOL = c_int;

    const ENABLE_VIRTUAL_TERMINAL_PROCESSING: DWORD = 0x0004;
    const STD_OUTPUT_HANDLE: DWORD = 0xFFFFFFF5;
    const STD_ERROR_HANDLE: DWORD = 0xFFFFFFF4;
    const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;
    const FALSE: BOOL = 0;
    const TRUE: BOOL = 1;

    // This is the win32 console API, taken from the 'winapi' crate.
    unsafe extern "system" {
        fn GetStdHandle(nStdHandle: DWORD) -> HANDLE;
        fn GetConsoleMode(hConsoleHandle: HANDLE, lpMode: LPDWORD) -> BOOL;
        fn SetConsoleMode(hConsoleHandle: HANDLE, dwMode: DWORD) -> BOOL;
    }

    unsafe fn get_handle(handle_num: DWORD) -> Result<HANDLE, ()> {
        match unsafe { GetStdHandle(handle_num) } {
            handle if handle == INVALID_HANDLE_VALUE => Err(()),
            handle => Ok(handle),
        }
    }

    unsafe fn enable_vt(handle: HANDLE) -> Result<(), ()> {
        let mut dw_mode: DWORD = 0;
        if unsafe { GetConsoleMode(handle, &mut dw_mode) } == FALSE {
            return Err(());
        }

        dw_mode |= ENABLE_VIRTUAL_TERMINAL_PROCESSING;
        match unsafe { SetConsoleMode(handle, dw_mode) } {
            result if result == TRUE => Ok(()),
            _ => Err(()),
        }
    }

    unsafe fn enable_ascii_colors_raw() -> Result<bool, ()> {
        let stdout_handle = unsafe { get_handle(STD_OUTPUT_HANDLE)? };
        let stderr_handle = unsafe { get_handle(STD_ERROR_HANDLE)? };

        unsafe { enable_vt(stdout_handle)? };
        if stdout_handle != stderr_handle {
            unsafe { enable_vt(stderr_handle)? };
        }

        Ok(true)
    }

    #[inline]
    pub fn enable_ascii_colors() -> bool {
        unsafe { enable_ascii_colors_raw().unwrap_or(false) }
    }
}

#[cfg(not(windows))]
mod windows_console {
    pub fn enable_ascii_colors() -> bool {
        true
    }
}

/// Enables ASCII terminal escape sequences on Windows consoles when
/// possible. Returns `true` if escape sequence support was successfully
/// enabled and `false` otherwise. On non-Windows targets, this method
/// always returns `true`.
///
/// Support for escape sequences in Windows consoles was added in the
/// Windows 10 anniversary update. For targets with older Windows
/// installations, this method is expected to return `false`.
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
        Ok(o) if accept(&o) => windows_console::enable_ascii_colors(),
        _ => false,
    }
}

fn accept(s: &str) -> bool {
    if s.eq("1") {
        true
    }
    else if s.eq_ignore_ascii_case("true") {
        true
    }
    else {
        false
    }
}
