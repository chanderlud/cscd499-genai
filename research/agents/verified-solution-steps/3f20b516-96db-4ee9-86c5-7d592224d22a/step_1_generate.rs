use std::path::Path;
use windows::core::{Result, Error, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, WriteFile, FILE_APPEND_DATA, FILE_ATTRIBUTE_NORMAL,
    FILE_SHARE_READ, OPEN_ALWAYS,
};
use windows::Win32::System::SystemServices::GENERIC_WRITE;

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

pub fn append_all(path: &Path, data: &[u8]) -> Result<()> {
    let wide_path = wide_null(path.as_os_str());
    let handle = unsafe {
        CreateFileW(
            PCWSTR(wide_path.as_ptr()),
            FILE_APPEND_DATA.0 | GENERIC_WRITE.0,
            FILE_SHARE_READ,
            None,
            OPEN_ALWAYS,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )
    };

    if handle == INVALID_HANDLE_VALUE {
        return Err(Error::from_win32());
    }

    let handle = Handle(handle);

    if data.is_empty() {
        return Ok(());
    }

    let mut bytes_written = 0u32;
    let success = unsafe {
        WriteFile(
            handle.0,
            Some(data.as_ptr() as *const _),
            data.len() as u32,
            Some(&mut bytes_written),
            None,
        )
    };

    if !success.as_bool() {
        return Err(Error::from_win32());
    }

    Ok(())
}

struct Handle(HANDLE);

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}