use std::path::Path;
use windows::core::{Result, Error, PCWSTR};
use windows::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE, CloseHandle};
use windows::Win32::Storage::FileSystem::{CreateFileW, GetFileSizeEx, FILE_GENERIC_READ, FILE_SHARE_READ, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

pub fn file_size(path: &Path) -> Result<u64> {
    let wide_path = wide_null(path.as_os_str());
    let handle = unsafe {
        CreateFileW(
            PCWSTR(wide_path.as_ptr()),
            FILE_GENERIC_READ.0,
            FILE_SHARE_READ,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )
    };

    if handle == INVALID_HANDLE_VALUE {
        return Err(Error::from_thread());
    }

    let mut size = 0i64;
    let result = unsafe { GetFileSizeEx(handle, &mut size) };
    
    // Always close the handle, even if GetFileSizeEx fails
    unsafe { CloseHandle(handle) };
    
    if result.as_bool() {
        Ok(size as u64)
    } else {
        Err(Error::from_thread())
    }
}