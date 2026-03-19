use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Result, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, ReadFile, FILE_FLAGS_AND_ATTRIBUTES, FILE_GENERIC_READ, FILE_SHARE_READ,
    OPEN_EXISTING,
};

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

pub fn file_checksum(path: &str) -> Result<u8> {
    let wide_path = wide_null(OsStr::new(path));

    // SAFETY: We're calling CreateFileW with valid parameters and checking the result.
    let handle = unsafe {
        CreateFileW(
            PCWSTR::from_raw(wide_path.as_ptr()),
            FILE_GENERIC_READ.0,
            FILE_SHARE_READ,
            None,
            OPEN_EXISTING,
            FILE_FLAGS_AND_ATTRIBUTES(0),
            None,
        )?
    };

    // Ensure handle is closed even if we return early
    let _guard = HandleGuard(handle);

    let mut total: u32 = 0;
    let mut buffer = [0u8; 4096];

    loop {
        let mut bytes_read = 0u32;

        // SAFETY: We're calling ReadFile with a valid handle and buffer.
        // The buffer pointer and size are valid for the duration of the call.
        unsafe {
            ReadFile(
                handle,
                Some(&mut buffer),
                Some(&mut bytes_read as *mut u32),
                None,
            )?;
        }

        if bytes_read == 0 {
            break; // End of file
        }

        // Process the chunk
        for &byte in &buffer[..bytes_read as usize] {
            total = total.wrapping_add(byte as u32);
        }
    }

    Ok((total % 256) as u8)
}

struct HandleGuard(HANDLE);

impl Drop for HandleGuard {
    fn drop(&mut self) {
        // SAFETY: We're closing a valid handle that was opened by CreateFileW.
        // The handle is guaranteed to be valid when this guard is created.
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}
