use windows::core::{Error, Result, PSTR};
use windows::Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER, ERROR_INVALID_DATA};

pub fn write_ascii_upper(buf: PSTR, cap: usize, value: &str) -> Result<usize> {
    // Check if input is ASCII
    if !value.is_ascii() {
        return Err(Error::from_hresult(ERROR_INVALID_DATA.to_hresult()));
    }

    // Check buffer capacity (need space for string + null terminator)
    if cap < value.len() + 1 {
        return Err(Error::from_hresult(ERROR_INSUFFICIENT_BUFFER.to_hresult()));
    }

    // SAFETY: We've validated that:
    // 1. The buffer has sufficient capacity (cap >= value.len() + 1)
    // 2. The caller is responsible for ensuring buf points to valid memory of at least cap bytes
    // 3. We only write within the bounds of the allocated buffer
    unsafe {
        let src = value.as_bytes();
        let dst = std::slice::from_raw_parts_mut(buf.0, cap);

        // Convert and copy each byte
        for (i, &byte) in src.iter().enumerate() {
            dst[i] = if byte.is_ascii_lowercase() {
                byte - 32 // Convert a-z to A-Z
            } else {
                byte
            };
        }

        // Add null terminator
        dst[value.len()] = 0;
    }

    Ok(value.len())
}