use windows::core::{Error, Result, PWSTR};
use windows::Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER, E_INVALIDARG};

pub fn write_wide(buf: PWSTR, cap: usize, value: &str) -> Result<usize> {
    // Check for null buffer pointer
    if buf.is_null() {
        return Err(Error::from_hresult(E_INVALIDARG));
    }

    // Convert the UTF-8 string to UTF-16
    let utf16: Vec<u16> = value.encode_utf16().collect();
    let len = utf16.len();

    // Check if buffer has enough capacity (including null terminator)
    if cap < len + 1 {
        return Err(Error::from_hresult(ERROR_INSUFFICIENT_BUFFER.to_hresult()));
    }

    // SAFETY: We've validated the buffer is non-null and has sufficient capacity.
    // The caller guarantees the buffer remains valid for the duration of this call.
    unsafe {
        // Copy the UTF-16 code units into the buffer
        std::ptr::copy_nonoverlapping(utf16.as_ptr(), buf.as_ptr(), len);

        // Write the null terminator
        *buf.as_ptr().add(len) = 0;
    }

    Ok(len)
}
