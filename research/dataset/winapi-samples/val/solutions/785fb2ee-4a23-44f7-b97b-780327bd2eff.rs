use std::os::windows::ffi::OsStrExt;
use windows::core::{Error, Result};
use windows::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS, HANDLE};
use windows::Win32::System::Threading::CreateMutexW;

pub fn create_named_mutex(name: &str) -> Result<(HANDLE, bool)> {
    // Convert to wide string using stack buffer (max 260 chars + null terminator)
    let mut wide_name = [0u16; 261];
    let mut i = 0;

    // Encode the string to UTF-16 and copy to buffer
    for u in std::ffi::OsStr::new(name).encode_wide() {
        if i >= 260 {
            // Name too long - return error
            return Err(Error::from_hresult(
                windows::core::HRESULT::from_win32(123), // ERROR_INVALID_NAME
            ));
        }
        wide_name[i] = u;
        i += 1;
    }

    // Null-terminate
    wide_name[i] = 0;

    // Create/open the mutex
    // SAFETY: We're passing a valid null-terminated wide string pointer
    let handle = unsafe { CreateMutexW(None, false, windows::core::PCWSTR(wide_name.as_ptr())) }?;

    // Check if mutex already existed
    // SAFETY: GetLastError is always safe to call
    let last_error = unsafe { GetLastError() };
    let created = last_error != ERROR_ALREADY_EXISTS;

    Ok((handle, created))
}
