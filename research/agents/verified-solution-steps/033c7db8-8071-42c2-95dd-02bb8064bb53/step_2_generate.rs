use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE};
use windows::Win32::System::Threading::CreateMutexW;

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

pub fn named_mutex_already_exists(name: &str) -> std::io::Result<bool> {
    let wide_name = wide_null(OsStr::new(name));
    let wide_name_pcwstr = PCWSTR(wide_name.as_ptr());

    // SAFETY: CreateMutexW is called with valid parameters and we check the result
    let handle = unsafe { CreateMutexW(None, false, wide_name_pcwstr) }?;

    // Check if the mutex already existed
    let already_exists = if handle != HANDLE::default() {
        // Get the last error BEFORE closing the handle
        let last_error = unsafe { GetLastError() };
        
        // SAFETY: We just obtained a valid handle from CreateMutexW
        unsafe { CloseHandle(handle) }?;
        
        // Compare the error code directly
        last_error == ERROR_ALREADY_EXISTS
    } else {
        false
    };

    Ok(already_exists)
}