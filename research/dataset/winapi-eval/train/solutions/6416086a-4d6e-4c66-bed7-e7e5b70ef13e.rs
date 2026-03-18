use std::path::Path;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_APPEND_DATA, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_DELETE, FILE_SHARE_READ,
    FILE_SHARE_WRITE, OPEN_ALWAYS, WriteFile,
};
use windows::Win32::System::IO::OVERLAPPED;
use windows::core::{Error, Result};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    s.encode_wide().chain(once(0)).collect()
}

pub fn append_all(path: &Path, data: &[u8]) -> Result<()> {
    let wide_path = wide_null(path.as_os_str());

    let handle = unsafe {
        CreateFileW(
            windows::core::PCWSTR(wide_path.as_ptr()),
            FILE_APPEND_DATA.0,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            None,
            OPEN_ALWAYS,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )
    }?;

    let result = write_all(handle, data);

    unsafe {
        CloseHandle(handle)?;
    }

    result
}

fn write_all(handle: HANDLE, data: &[u8]) -> Result<()> {
    let mut bytes_written = 0u32;
    let mut remaining = data;

    while !remaining.is_empty() {
        // SAFETY: WriteFile is a valid Win32 API call with proper parameters
        let success = unsafe {
            WriteFile(
                handle,
                Some(remaining),
                Some(&mut bytes_written as *mut u32),
                Some(std::ptr::null_mut::<OVERLAPPED>()),
            )
        };

        success?;

        if bytes_written == 0 {
            // No progress made, avoid infinite loop
            return Err(Error::from_thread());
        }

        remaining = &remaining[bytes_written as usize..];
    }

    Ok(())
}
