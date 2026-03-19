use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::{ERROR_ALREADY_EXISTS, ERROR_INVALID_PARAMETER, HANDLE};
use windows::Win32::System::Threading::CreateEventW;

/// Converts a string to a wide (UTF-16) null-terminated string in a fixed-size buffer.
/// Returns an error if the string would exceed MAX_PATH (260) wide characters.
fn to_wide_fixed(s: &str, buffer: &mut [u16; 261]) -> Result<()> {
    // Check if the string would exceed MAX_PATH (260) wide characters
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

/// Creates or opens a named manual-reset event with initial state false.
/// Returns a tuple of (handle, created) where created is true if this call created the event.
pub fn create_named_event(name: &str) -> Result<(HANDLE, bool)> {
    let mut wide_buffer = [0u16; 261];
    to_wide_fixed(name, &mut wide_buffer)?;

    // Create manual-reset event with initial state false
    let handle = unsafe {
        CreateEventW(
            None,                         // Default security attributes
            true,                         // Manual-reset event
            false,                        // Initial state: non-signaled
            PCWSTR(wide_buffer.as_ptr()), // Event name
        )
    }?;

    // Check if the event already existed
    // Note: CreateEventW returns a valid handle even when ERROR_ALREADY_EXISTS is set
    let already_exists = Error::from_thread().code() == HRESULT::from_win32(ERROR_ALREADY_EXISTS.0);

    Ok((handle, !already_exists))
}
