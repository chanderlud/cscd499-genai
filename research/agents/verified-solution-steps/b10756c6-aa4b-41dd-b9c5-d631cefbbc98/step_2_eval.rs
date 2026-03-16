use std::path::Path;
use windows::core::{Result, Error, PCWSTR};
use windows::Win32::Foundation::{FILETIME, HANDLE, CloseHandle};
use windows::Win32::Storage::FileSystem::{CreateFileW, SetFileTime, FILE_WRITE_ATTRIBUTES, FILE_SHARE_READ, OPEN_EXISTING, FILE_FLAG_BACKUP_SEMANTICS};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    s.encode_wide().chain(once(0)).collect()
}

pub fn set_last_write_time(path: &Path, ft: FILETIME) -> Result<()> {
    let wide_path = wide_null(path.as_os_str());
    
    let handle = unsafe {
        CreateFileW(
            PCWSTR::from_raw(wide_path.as_ptr()),
            FILE_WRITE_ATTRIBUTES.0,
            FILE_SHARE_READ,
            None,
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS,
            None,
        )?
    };
    
    let result = unsafe {
        SetFileTime(
            handle,
            None,
            None,
            Some(&ft as *const FILETIME),
        )
    };
    
    unsafe { CloseHandle(handle) }?;
    
    result
}