use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, GetFinalPathNameByHandleW, FILE_FLAG_BACKUP_SEMANTICS, FILE_SHARE_READ,
    OPEN_EXISTING,
};
use windows::Win32::System::SystemServices::GENERIC_READ;

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

pub fn final_path_from_handle(path: &Path) -> std::io::Result<String> {
    // Convert path to wide string for Win32 API
    let wide_path = wide_null(path.as_os_str());
    
    // Open file handle with minimal required access
    let handle = unsafe {
        CreateFileW(
            PCWSTR(wide_path.as_ptr()),
            GENERIC_READ.0,
            FILE_SHARE_READ,
            None,
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS,
            None,
        )
    };
    
    // Check if handle creation failed
    if handle == INVALID_HANDLE_VALUE {
        return Err(Error::from_thread().into());
    }
    
    // Ensure handle gets closed even if we return early
    struct HandleGuard(HANDLE);
    impl Drop for HandleGuard {
        fn drop(&mut self) {
            unsafe { CloseHandle(self.0) };
        }
    }
    let _guard = HandleGuard(handle);
    
    // First call to get required buffer size
    let required_len = unsafe {
        GetFinalPathNameByHandleW(handle, None, 0)
    };
    
    if required_len == 0 {
        return Err(Error::from_thread().into());
    }
    
    // Allocate buffer with required size (including null terminator)
    let mut buffer = vec![0u16; required_len as usize];
    
    // Second call to actually get the path
    let written_len = unsafe {
        GetFinalPathNameByHandleW(handle, Some(buffer.as_mut_slice()), required_len)
    };
    
    if written_len == 0 {
        return Err(Error::from_thread().into());
    }
    
    // Convert UTF-16 buffer to String
    String::from_utf16(&buffer[..written_len as usize])
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}