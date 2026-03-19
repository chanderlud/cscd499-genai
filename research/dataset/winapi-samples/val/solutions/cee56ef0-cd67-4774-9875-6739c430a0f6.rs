use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::{ERROR_ALREADY_EXISTS, ERROR_INVALID_PARAMETER, HANDLE};
use windows::Win32::System::Threading::CreateMutexW;

/// Converts a string to a null-terminated wide string in a fixed-size buffer.
/// Returns an error if the string is too long (>= 260 characters).
fn to_wide_fixed(s: &str, buffer: &mut [u16; 261]) -> Result<()> {
    // Check if the string would exceed MAX_PATH (260) wide characters
    // We need to count UTF-16 code units without allocating
    let mut utf16_len = 0;
    for _ in s.encode_utf16() {
        utf16_len += 1;
        if utf16_len > 260 {
            return Err(Error::from_hresult(HRESULT::from_win32(
                ERROR_INVALID_PARAMETER.0,
            )));
        }
    }

    // Convert to wide string in the buffer
    let mut i = 0;
    for c in s.encode_utf16() {
        buffer[i] = c;
        i += 1;
    }
    buffer[i] = 0; // Null terminator

    Ok(())
}

/// Creates or opens a named mutex and returns whether this call created it.
///
/// # Arguments
/// * `name` - The name of the mutex (max 260 characters)
///
/// # Returns
/// A tuple containing the mutex handle and a boolean indicating if the mutex was created (true)
/// or already existed (false).
pub fn create_named_mutex(name: &str) -> Result<(HANDLE, bool)> {
    // Fixed-size buffer for the wide string (260 chars + null terminator)
    let mut wide_buffer = [0u16; 261];

    // Convert the name to wide string in the fixed buffer
    to_wide_fixed(name, &mut wide_buffer)?;

    // Create or open the mutex
    // SAFETY: We're calling a Windows API with a valid null-terminated wide string
    let handle = unsafe { CreateMutexW(None, false, PCWSTR(wide_buffer.as_ptr())) }?;

    // Check if the mutex already existed
    let already_exists = Error::from_thread().code() == HRESULT::from_win32(ERROR_ALREADY_EXISTS.0);

    Ok((handle, !already_exists))
}
