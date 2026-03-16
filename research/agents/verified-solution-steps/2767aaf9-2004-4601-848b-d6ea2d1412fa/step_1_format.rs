use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows::core::{Error, Result, HRESULT, PCWSTR};

pub fn collect_wide_strings(values: &[PCWSTR]) -> Result<Vec<String>> {
    let mut result = Vec::with_capacity(values.len());

    for &pcwstr in values {
        // Check for null pointer
        if pcwstr.is_null() {
            return Err(Error::from_hresult(HRESULT::from_win32(0x80070057))); // E_INVALIDARG
        }

        // Find the length of the null-terminated string
        let mut len = 0;
        // SAFETY: We've checked for null above, and we trust the caller that the pointer
        // points to a valid null-terminated UTF-16 string
        unsafe {
            while *pcwstr.0.add(len) != 0 {
                len += 1;
            }
        }

        // Convert to OsString then to String
        // SAFETY: We've calculated the length and verified it's null-terminated
        let wide_slice = unsafe { std::slice::from_raw_parts(pcwstr.0, len) };
        let os_string = OsString::from_wide(wide_slice);

        match os_string.into_string() {
            Ok(s) => result.push(s),
            Err(_) => {
                return Err(Error::from_hresult(HRESULT::from_win32(0x8007000D)));
                // ERROR_INVALID_DATA
            }
        }
    }

    Ok(result)
}