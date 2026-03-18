use windows::core::{Error, Result, HRESULT, PCWSTR};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

fn pcwstr_to_string(pcwstr: PCWSTR) -> Result<String> {
    // SAFETY: We check for null pointer before calling this function
    let ptr = pcwstr.0;
    if ptr.is_null() {
        return Ok(String::new());
    }

    // Find the length of the null-terminated UTF-16 string
    let mut len = 0;
    // SAFETY: We've checked ptr is not null, and we stop at the null terminator
    while unsafe { *ptr.add(len) } != 0 {
        len += 1;
    }

    // Convert the UTF-16 slice to a Rust String
    // SAFETY: We've calculated the correct length and verified null termination
    let wide_slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    String::from_utf16(wide_slice).map_err(|_| {
        // Invalid UTF-16 sequence
        Error::from_hresult(HRESULT::from_win32(0x8007000D)) // E_INVALIDARG
    })
}

fn is_non_empty(pcwstr: PCWSTR) -> bool {
    // SAFETY: We only dereference after checking for null
    !pcwstr.0.is_null() && unsafe { *pcwstr.0 } != 0
}

pub fn choose_wide(primary: Option<PCWSTR>, fallback: Option<PCWSTR>) -> Result<String> {
    // Check primary first
    if let Some(p) = primary {
        if is_non_empty(p) {
            return pcwstr_to_string(p);
        }
    }

    // Check fallback if primary was missing or empty
    if let Some(f) = fallback {
        if is_non_empty(f) {
            return pcwstr_to_string(f);
        }
    }

    // Both were missing or empty
    Ok(String::new())
}
