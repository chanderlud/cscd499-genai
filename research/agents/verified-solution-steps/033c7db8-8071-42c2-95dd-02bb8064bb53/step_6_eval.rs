use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE};
use windows::Win32::System::Threading::CreateMutexW;

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

fn is_valid_mutex_name(name: &str) -> bool {
    // Mutex names cannot contain backslash except as first character for namespace
    // Also cannot contain null terminator (but Rust strings don't have nulls)
    for (i, c) in name.chars().enumerate() {
        if c == '\\' && i != 0 {
            return false;
        }
        if c == '\0' {
            return false;
        }
    }
    true
}

pub fn named_mutex_already_exists(name: &str) -> std::io::Result<bool> {
    // Validate input
    if name.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Mutex name cannot be empty",
        ));
    }
    
    // Check for invalid characters
    if !is_valid_mutex_name(name) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Mutex name contains invalid characters",
        ));
    }
    
    let wide_name = wide_null(OsStr::new(name));
    
    // Check length (MAX_PATH is 260, but mutex names have different limits)
    // Mutex names are limited to MAX_PATH (260) characters including null terminator
    if wide_name.len() > 260 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Mutex name is too long",
        ));
    }
    
    let wide_name_pcwstr = PCWSTR(wide_name.as_ptr());

    // SAFETY: CreateMutexW is called with valid parameters and we check the result
    let handle = unsafe { CreateMutexW(None, false, wide_name_pcwstr) }
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    // Check if the mutex already existed
    let already_exists = if handle != HANDLE::default() {
        // Get the last error BEFORE closing the handle
        let last_error = unsafe { GetLastError() };
        
        // SAFETY: We just obtained a valid handle from CreateMutexW
        unsafe { CloseHandle(handle) }
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        // Compare the error code directly
        last_error == ERROR_ALREADY_EXISTS
    } else {
        false
    };

    Ok(already_exists)
}