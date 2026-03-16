use std::path::{Path, PathBuf};
use windows::core::{Result, Error, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, GetFinalPathNameByHandleW, FILE_FLAG_BACKUP_SEMANTICS,
    FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

pub fn final_path(path: &Path) -> Result<PathBuf> {
    let wide_path = wide_null(path.as_os_str());
    
    // Open the file with minimal access and backup semantics (required for directories)
    let handle = unsafe {
        CreateFileW(
            PCWSTR(wide_path.as_ptr()),
            0, // No access needed, just for querying
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            None,
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS,
            None,
        )?
    };
    
    if handle == INVALID_HANDLE_VALUE {
        // This shouldn't happen since CreateFileW returns Result, but just in case
        return Err(Error::from_thread());
    }
    
    // Ensure handle gets closed even if we return early
    struct HandleGuard(HANDLE);
    impl Drop for HandleGuard {
        fn drop(&mut self) {
            unsafe { let _ = CloseHandle(self.0); }
        }
    }
    let _guard = HandleGuard(handle);
    
    // First call to get required buffer size
    let required_len = unsafe {
        GetFinalPathNameByHandleW(handle, None, 0)
    };
    
    if required_len == 0 {
        return Err(Error::from_thread());
    }
    
    // Allocate buffer and get the actual path
    let mut buffer = vec![0u16; required_len as usize];
    let written = unsafe {
        GetFinalPathNameByHandleW(handle, Some(&mut buffer), 0)
    };
    
    if written == 0 || written > required_len {
        return Err(Error::from_thread());
    }
    
    // Convert to PathBuf, trimming any trailing nulls
    let path_str = String::from_utf16_lossy(&buffer[..written as usize]);
    Ok(PathBuf::from(path_str))
}