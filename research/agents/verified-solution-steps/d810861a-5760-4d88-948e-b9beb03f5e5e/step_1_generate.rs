use windows::core::{PCWSTR, Result};
use windows::Win32::Foundation::ERROR_INVALID_PARAMETER;

pub fn pcwstr_to_string(ptr: PCWSTR) -> Result<String> {
    // Handle null pointer case
    if ptr.is_null() {
        return Ok(String::new());
    }

    // SAFETY: We've checked for null, and we'll read until we find a null terminator.
    // The caller is responsible for ensuring the pointer is valid and null-terminated.
    let mut len = 0;
    let raw_ptr = ptr.as_ptr();
    
    // Find the length by scanning for null terminator
    loop {
        // SAFETY: Reading from the pointer at offset `len`. We'll stop at null.
        let value = unsafe { *raw_ptr.add(len) };
        if value == 0 {
            break;
        }
        len += 1;
    }

    // SAFETY: We've determined the length, and the pointer is valid for `len` elements.
    let utf16_slice = unsafe { std::slice::from_raw_parts(raw_ptr, len) };
    
    String::from_utf16(utf16_slice).map_err(|_| {
        // Convert invalid UTF-16 to an appropriate Windows error
        let hr = ERROR_INVALID_PARAMETER.to_hresult();
        windows::core::Error::from_hresult(hr)
    })
}