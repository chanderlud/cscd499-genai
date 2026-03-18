use std::path::Path;
use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::{
    CloseHandle, ERROR_WRITE_FAULT, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE,
};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, WriteFile, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_NONE,
};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

pub fn write_all(path: &Path, data: &[u8]) -> Result<()> {
    let wide_path = wide_null(path.as_os_str());

    let handle = unsafe {
        CreateFileW(
            PCWSTR::from_raw(wide_path.as_ptr()),
            GENERIC_WRITE.0,
            FILE_SHARE_NONE,
            None,
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )
    }?;

    if handle == INVALID_HANDLE_VALUE {
        return Err(Error::from_thread());
    }

    let result = write_all_bytes(handle, data);

    unsafe {
        let _ = CloseHandle(handle);
    }

    result
}

fn write_all_bytes(handle: HANDLE, data: &[u8]) -> Result<()> {
    let mut bytes_written = 0u32;
    let mut remaining = data;

    while !remaining.is_empty() {
        unsafe {
            WriteFile(
                handle,
                Some(remaining),
                Some(&mut bytes_written as *mut u32),
                None,
            )
        }?;

        if bytes_written == 0 {
            return Err(Error::from_hresult(ERROR_WRITE_FAULT.to_hresult()));
        }

        remaining = &remaining[bytes_written as usize..];
    }

    Ok(())
}
