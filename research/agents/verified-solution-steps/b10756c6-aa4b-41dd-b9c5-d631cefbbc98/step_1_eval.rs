use std::path::Path;
use windows::core::{Result, Error, HRESULT};
use windows::Win32::Foundation::{FILETIME, HANDLE, CloseHandle, INVALID_HANDLE_VALUE};
use windows::Win32::Storage::FileSystem::{CreateFileW, SetFileTime, FILE_WRITE_ATTRIBUTES, FILE_SHARE_READ, OPEN_EXISTING, FILE_FLAG_BACKUP_SEMANTICS};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

pub fn set_last_write_time(path: &Path, ft: FILETIME) -> Result<()> {
    let wide_path = wide_null(path.as_os_str());
    
    // SAFETY: We're calling a Windows API function with valid parameters.
    // The wide_path is null-terminated and valid for the duration of the call.
    let handle = unsafe {
        CreateFileW(
            wide_path.as_ptr(),
            FILE_WRITE_ATTRIBUTES.0,
            FILE_SHARE_READ,
            std::ptr::null(),
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS,
            HANDLE::default(),
        )
    };
    
    if handle == INVALID_HANDLE_VALUE {
        return Err(Error::from_thread());
    }
    
    // Ensure handle gets closed even if SetFileTime fails
    let result = (|| -> Result<()> {
        // SAFETY: We have a valid handle from CreateFileW and are passing valid pointers.
        // The ft pointer is valid for the duration of the call.
        let success = unsafe {
            SetFileTime(
                handle,
                std::ptr::null(), // creation time - leave unchanged
                std::ptr::null(), // last access time - leave unchanged
                &ft,
            )
        };
        
        if !success.as_bool() {
            return Err(Error::from_thread());
        }
        
        Ok(())
    })();
    
    // SAFETY: We're closing a valid handle that we own.
    unsafe { CloseHandle(handle) };
    
    result
}