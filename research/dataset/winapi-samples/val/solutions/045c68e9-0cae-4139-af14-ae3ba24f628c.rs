use windows::core::{Error, Result};
use windows::Win32::Foundation::{ERROR_ALREADY_EXISTS, E_INVALIDARG, HANDLE};
use windows::Win32::System::Threading::CreateMutexW;

pub fn create_named_mutex_no_alloc(name: &str) -> Result<(HANDLE, bool)> {
    // Convert to wide string using stack buffer
    let mut wide_buf = [0u16; 260];
    let mut i = 0;
    for u in name.encode_utf16() {
        if i >= wide_buf.len() - 1 {
            return Err(Error::from_hresult(E_INVALIDARG)); // Fixed line
        }
        wide_buf[i] = u;
        i += 1;
    }
    wide_buf[i] = 0; // Null terminator

    // Create the mutex
    let handle = unsafe { CreateMutexW(None, false, windows::core::PCWSTR(wide_buf.as_ptr()))? };

    // Check if it already existed
    let last_error = unsafe { windows::Win32::Foundation::GetLastError() };
    let created = last_error != ERROR_ALREADY_EXISTS;

    Ok((handle, created))
}
