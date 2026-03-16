use std::path::Path;
use windows::core::{Result, Error, HRESULT};
use windows::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE, ERROR_HANDLE_EOF, CloseHandle};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, ReadFile, FILE_FLAG_SEQUENTIAL_SCAN, FILE_SHARE_READ, OPEN_EXISTING,
    FILE_ATTRIBUTE_NORMAL,
};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

pub fn read_to_end(path: &Path) -> Result<Vec<u8>> {
    let wide_path = wide_null(path.as_os_str());
    
    // SAFETY: We're calling a Win32 API with valid parameters.
    let handle = unsafe {
        CreateFileW(
            wide_path.as_ptr(),
            0x80000000, // GENERIC_READ
            FILE_SHARE_READ,
            None,
            OPEN_EXISTING,
            FILE_FLAG_SEQUENTIAL_SCAN | FILE_ATTRIBUTE_NORMAL,
            None,
        )
    }?;

    if handle == INVALID_HANDLE_VALUE {
        return Err(Error::from_hresult(HRESULT::from_win32(
            windows::Win32::Foundation::GetLastError().0,
        )));
    }

    let mut buffer = Vec::new();
    let mut temp_buf = [0u8; 4096];
    let mut bytes_read = 0u32;

    loop {
        // SAFETY: We're calling ReadFile with a valid handle and buffer.
        let success = unsafe {
            ReadFile(
                handle,
                temp_buf.as_mut_ptr() as *mut _,
                temp_buf.len() as u32,
                &mut bytes_read,
                None,
            )
        };

        if !success.as_bool() {
            let err = windows::Win32::Foundation::GetLastError();
            if err == ERROR_HANDLE_EOF {
                break;
            }
            // SAFETY: We're closing the handle before returning the error.
            unsafe { CloseHandle(handle) };
            return Err(Error::from_hresult(HRESULT::from_win32(err.0)));
        }

        if bytes_read == 0 {
            break;
        }

        buffer.extend_from_slice(&temp_buf[..bytes_read as usize]);
    }

    // SAFETY: We're closing the handle after we're done with it.
    unsafe { CloseHandle(handle) };
    Ok(buffer)
}