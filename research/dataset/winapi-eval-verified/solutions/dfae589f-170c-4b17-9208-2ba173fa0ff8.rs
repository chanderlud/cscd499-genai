use windows::core::Result;
use windows::Win32::Foundation::{ERROR_ALREADY_EXISTS, HANDLE};
use windows::Win32::System::Threading::CreateMutexW;

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

pub fn create_named_mutex(name: &str) -> Result<(HANDLE, bool)> {
    let wide_name = wide_null(std::ffi::OsStr::new(name));

    // SAFETY: CreateMutexW is called with valid parameters and we check the result
    let handle = unsafe { CreateMutexW(None, false, windows::core::PCWSTR(wide_name.as_ptr())) }?;

    // Check if the mutex already existed
    let last_error = unsafe { windows::Win32::Foundation::GetLastError() };
    let created = last_error != ERROR_ALREADY_EXISTS;

    Ok((handle, created))
}
