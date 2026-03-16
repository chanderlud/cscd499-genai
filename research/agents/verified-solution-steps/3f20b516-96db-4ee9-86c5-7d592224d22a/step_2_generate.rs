use std::path::Path;
use windows::core::{Result, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, WriteFile, FILE_APPEND_DATA, FILE_ATTRIBUTE_NORMAL,
    FILE_SHARE_READ, OPEN_ALWAYS,
};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

pub fn append_all(path: &Path, data: &[u8]) -> Result<()> {
    let wide_path = wide_null(path.as_os_str());
    let handle = unsafe {
        CreateFileW(
            PCWSTR(wide_path.as_ptr()),
            FILE_APPEND_DATA.0,
            FILE_SHARE_READ,
            None,
            OPEN_ALWAYS,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )
    }?;

    let handle = Handle(handle);

    if data.is_empty() {
        return Ok(());
    }

    let mut bytes_written = 0u32;
    unsafe {
        WriteFile(
            handle.0,
            Some(data),
            Some(&mut bytes_written as *mut u32),
            None,
        )
    }?;

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