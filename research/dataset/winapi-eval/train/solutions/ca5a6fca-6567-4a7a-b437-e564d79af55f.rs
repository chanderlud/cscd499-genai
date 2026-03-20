use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows::core::{Error, PCWSTR};
use windows::Win32::Foundation::GENERIC_READ;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, GetFinalPathNameByHandleW, FILE_FLAG_BACKUP_SEMANTICS, FILE_SHARE_READ,
    GETFINALPATHNAMEBYHANDLE_FLAGS, OPEN_EXISTING,
};

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

pub fn final_path_from_handle(path: &Path) -> std::io::Result<String> {
    let wide_path = wide_null(path.as_os_str());

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

    struct HandleGuard(HANDLE);
    impl Drop for HandleGuard {
        fn drop(&mut self) {
            unsafe {
                let _ = CloseHandle(self.0);
            }
        }
    }
    let _guard = HandleGuard(handle);

    let required_len =
        unsafe { GetFinalPathNameByHandleW(handle, &mut [], GETFINALPATHNAMEBYHANDLE_FLAGS(0)) };

    if required_len == 0 {
        return Err(Error::from_thread().into());
    }

    let mut buffer = vec![0u16; required_len as usize];

    let written_len = unsafe {
        GetFinalPathNameByHandleW(handle, &mut buffer, GETFINALPATHNAMEBYHANDLE_FLAGS(0))
    };

    if written_len == 0 {
        return Err(Error::from_thread().into());
    }

    let path_str = String::from_utf16(&buffer[..written_len as usize])
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let is_root = path_str.ends_with('\\') && {
        let without_trailing = &path_str[..path_str.len() - 1];
        without_trailing.ends_with(':')
            || (path_str.starts_with("\\\\?\\") && path_str.matches('\\').count() == 3)
            || (path_str.starts_with("\\\\")
                && !path_str.starts_with("\\\\?\\")
                && path_str.matches('\\').count() == 3)
    };

    if is_root {
        Ok(path_str[..path_str.len() - 1].to_string())
    } else {
        Ok(path_str)
    }
}
