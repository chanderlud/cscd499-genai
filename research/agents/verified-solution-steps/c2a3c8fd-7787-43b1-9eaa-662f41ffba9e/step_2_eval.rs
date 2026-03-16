use std::path::{Path, PathBuf};
use windows::core::{Result, Error, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, GetFinalPathNameByHandleW, FILE_FLAG_BACKUP_SEMANTICS,
    FILE_SHARE_DELETE, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
    GETFINALPATHNAMEBYHANDLE_FLAGS,
};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    s.encode_wide().chain(once(0)).collect()
}

pub fn final_path(path: &Path) -> Result<PathBuf> {
    let wide_path = wide_null(path.as_os_str());
    
    let handle = unsafe {
        CreateFileW(
            PCWSTR(wide_path.as_ptr()),
            0,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            None,
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS,
            None,
        )?
    };
    
    if handle == INVALID_HANDLE_VALUE {
        return Err(Error::from_thread());
    }
    
    struct HandleGuard(HANDLE);
    impl Drop for HandleGuard {
        fn drop(&mut self) {
            unsafe { let _ = CloseHandle(self.0); }
        }
    }
    let _guard = HandleGuard(handle);
    
    // First call: get required buffer size
    let required_len = unsafe {
        GetFinalPathNameByHandleW(handle, &mut [], GETFINALPATHNAMEBYHANDLE_FLAGS(0))
    };
    
    if required_len == 0 {
        return Err(Error::from_thread());
    }
    
    // Allocate buffer and get the actual path
    let mut buffer = vec![0u16; required_len as usize];
    let written = unsafe {
        GetFinalPathNameByHandleW(handle, &mut buffer, GETFINALPATHNAMEBYHANDLE_FLAGS(0))
    };
    
    if written == 0 || written > required_len {
        return Err(Error::from_thread());
    }
    
    let path_str = String::from_utf16_lossy(&buffer[..written as usize]);
    Ok(PathBuf::from(path_str))
}