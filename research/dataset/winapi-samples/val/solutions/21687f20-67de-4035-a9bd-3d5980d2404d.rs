use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::{ERROR_NO_UNICODE_TRANSLATION, E_POINTER};
use windows::Win32::Globalization::{
    MultiByteToWideChar, WideCharToMultiByte, CP_UTF8, MB_ERR_INVALID_CHARS, WC_ERR_INVALID_CHARS,
};

pub fn to_wide_null(s: &str) -> Result<Vec<u16>> {
    if s.is_empty() {
        return Ok(vec![0]);
    }

    let utf8_bytes = s.as_bytes();

    // First call: get required buffer size
    let required_size =
        unsafe { MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, utf8_bytes, None) };

    if required_size == 0 {
        return Err(Error::from_thread());
    }

    // Allocate buffer with space for null terminator
    let mut wide_chars = vec![0u16; required_size as usize + 1];

    // Second call: perform actual conversion
    let chars_written = unsafe {
        MultiByteToWideChar(
            CP_UTF8,
            MB_ERR_INVALID_CHARS,
            utf8_bytes,
            Some(&mut wide_chars[..required_size as usize]),
        )
    };

    if chars_written == 0 {
        return Err(Error::from_thread());
    }

    // Ensure null terminator is present
    wide_chars[required_size as usize] = 0;

    Ok(wide_chars)
}

/// Converts a null-terminated UTF-16 string pointer to a Rust String.
///
/// # Safety
///
/// This function is unsafe because it dereferences a raw pointer. The caller must ensure that:
/// - `ptr` is a valid pointer to a null-terminated UTF-16 string
/// - The memory pointed to by `ptr` is valid for the entire duration of the call
/// - The UTF-16 string is properly null-terminated
/// - The pointer is properly aligned
///
/// If any of these conditions are violated, the behavior is undefined.
pub unsafe fn from_wide_ptr(ptr: *const u16) -> Result<String> {
    if ptr.is_null() {
        return Err(Error::from_hresult(E_POINTER));
    }

    // Find length of null-terminated string
    let mut len = 0;
    while unsafe { *ptr.add(len) } != 0 {
        len += 1;
    }

    // Convert to slice (excluding null terminator for conversion)
    let wide_slice = unsafe { std::slice::from_raw_parts(ptr, len) };

    // First call: get required buffer size
    let required_size =
        unsafe { WideCharToMultiByte(CP_UTF8, WC_ERR_INVALID_CHARS, wide_slice, None, None, None) };

    if required_size == 0 {
        return Err(Error::from_thread());
    }

    // Allocate buffer for UTF-8 bytes
    let mut utf8_bytes = vec![0u8; required_size as usize];

    // Second call: perform actual conversion
    let bytes_written = unsafe {
        WideCharToMultiByte(
            CP_UTF8,
            WC_ERR_INVALID_CHARS,
            wide_slice,
            Some(&mut utf8_bytes),
            None,
            None,
        )
    };

    if bytes_written == 0 {
        return Err(Error::from_thread());
    }

    // Convert to String, validating UTF-8
    String::from_utf8(utf8_bytes)
        .map_err(|_| Error::from_hresult(HRESULT::from_win32(ERROR_NO_UNICODE_TRANSLATION.0)))
}

fn main() -> Result<()> {
    let wide = to_wide_null("hello")?;
    println!("Wide: {:?}", wide);

    let utf8 = unsafe { from_wide_ptr(wide.as_ptr()) }?;
    println!("UTF-8: {}", utf8);

    Ok(())
}
