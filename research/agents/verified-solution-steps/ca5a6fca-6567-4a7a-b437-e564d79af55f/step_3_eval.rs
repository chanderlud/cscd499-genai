use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows::core::{Error, PCWSTR, Result};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, GetFinalPathNameByHandleW, FILE_FLAG_BACKUP_SEMANTICS, FILE_SHARE_READ,
    GETFINALPATHNAMEBYHANDLE_FLAGS, OPEN_EXISTING,
};
use windows::Win32::Foundation::GENERIC_READ;

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

pub fn final_path_from_handle(path: &Path) -> std::io::Result<String> {
    // Convert path to wide string for Win32 API
    let wide_path = wide_null(path.as_os_str());
    
    // Open file handle with minimal required access
    // SAFETY: CreateFileW is called with valid parameters
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
    }?;
    
    // Ensure handle gets closed even if we return early
    struct HandleGuard(HANDLE);
    impl Drop for HandleGuard {
        fn drop(&mut self) {
            // SAFETY: CloseHandle is called with a valid handle
            unsafe { CloseHandle(self.0) };
        }
    }
    let _guard = HandleGuard(handle);
    
    // First call to get required buffer size
    // SAFETY: GetFinalPathNameByHandleW is called with valid handle and empty buffer
    let required_len = unsafe {
        GetFinalPathNameByHandleW(handle, &mut [], GETFINALPATHNAMEBYHANDLE_FLAGS(0))
    };
    
    if required_len == 0 {
        return Err(Error::from_thread().into());
    }
    
    // Allocate buffer with required size (including null terminator)
    let mut buffer = vec![0u16; required_len as usize];
    
    // Second call to actually get the path
    // SAFETY: GetFinalPathNameByHandleW is called with valid handle and buffer
    let written_len = unsafe {
        GetFinalPathNameByHandleW(handle, &mut buffer, GETFINALPATHNAMEBYHANDLE_FLAGS(0))
    };
    
    if written_len == 0 {
        return Err(Error::from_thread().into());
    }
    
    // Convert UTF-16 buffer to String
    let path_str = String::from_utf16(&buffer[..written_len as usize])
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    
    // For root directories, GetFinalPathNameByHandleW may return a path with trailing backslash
    // The test expects the canonical path without trailing backslash for root directories
    // Check if this is a root directory path (like "C:\" or "\\?\C:\")
    let is_root = path_str.ends_with('\\') && {
        // Check if it's a drive root (C:\) or a UNC root (\\server\share\)
        let without_trailing = &path_str[..path_str.len() - 1];
        without_trailing.ends_with(':') || // Drive root like "C:"
        (path_str.starts_with("\\\\?\\") && path_str.matches('\\').count() == 3) || // \\?\C:\
        (path_str.starts_with("\\\\") && !path_str.starts_with("\\\\?\\") && path_str.matches('\\').count() == 3) // \\server\share
    };
    
    if is_root {
        // Remove trailing backslash for root directories
        Ok(path_str[..path_str.len() - 1].to_string())
    } else {
        Ok(path_str)
    }
}