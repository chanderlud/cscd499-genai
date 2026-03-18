use windows::core::{Error, Result, HRESULT, PWSTR};
use windows::Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER, E_INVALIDARG};

pub fn write_wide_noalloc(buf: PWSTR, cap: usize, value: &str) -> Result<usize> {
    // Check for null buffer
    if buf.0.is_null() {
        return Err(Error::from_hresult(E_INVALIDARG));
    }

    // First pass: count UTF-16 code units needed
    let mut required_len = 0;
    for _ in value.encode_utf16() {
        required_len += 1;
    }

    // Check if buffer is too small (need space for terminator)
    if cap < required_len + 1 {
        return Err(Error::from_hresult(HRESULT::from_win32(
            ERROR_INSUFFICIENT_BUFFER.0,
        )));
    }

    // Second pass: write UTF-16 code units to buffer
    let mut i = 0;
    for code_unit in value.encode_utf16() {
        // SAFETY: We've verified buf is non-null and we have enough capacity
        unsafe {
            *buf.0.add(i) = code_unit;
        }
        i += 1;
    }

    // Write null terminator
    // SAFETY: We've verified buf is non-null and we have enough capacity
    unsafe {
        *buf.0.add(i) = 0;
    }

    Ok(required_len)
}
