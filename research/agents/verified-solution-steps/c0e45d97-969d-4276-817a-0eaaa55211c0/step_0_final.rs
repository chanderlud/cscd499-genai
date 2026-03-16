use std::path::Path;
use windows::core::{Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, ERROR_HANDLE_EOF, GENERIC_READ};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, ReadFile, FILE_ATTRIBUTE_NORMAL, FILE_CREATION_DISPOSITION,
    FILE_FLAGS_AND_ATTRIBUTES, FILE_FLAG_SEQUENTIAL_SCAN, FILE_SHARE_MODE, FILE_SHARE_READ,
    OPEN_EXISTING,
};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    s.encode_wide().chain(once(0)).collect()
}

pub fn read_to_end(path: &Path) -> Result<Vec<u8>> {
    let wide_path = wide_null(path.as_os_str());

    // SAFETY: We're calling a Win32 API with valid parameters.
    let handle = unsafe {
        CreateFileW(
            PCWSTR::from_raw(wide_path.as_ptr()),
            GENERIC_READ.0,
            FILE_SHARE_READ,
            None,
            OPEN_EXISTING,
            FILE_FLAG_SEQUENTIAL_SCAN | FILE_ATTRIBUTE_NORMAL,
            None,
        )
    }?;

    let mut buffer = Vec::new();
    let mut temp_buf = [0u8; 4096];
    let mut bytes_read = 0u32;

    loop {
        // SAFETY: We're calling ReadFile with a valid handle and buffer.
        match unsafe { ReadFile(handle, Some(&mut temp_buf[..]), Some(&mut bytes_read), None) } {
            Ok(()) => {
                if bytes_read == 0 {
                    break;
                }
                buffer.extend_from_slice(&temp_buf[..bytes_read as usize]);
            }
            Err(e) => {
                if e.code() == HRESULT::from_win32(ERROR_HANDLE_EOF.0) {
                    break;
                }
                // SAFETY: We're closing the handle before returning the error.
                unsafe {
                    let _ = CloseHandle(handle);
                };
                return Err(e);
            }
        }
    }

    // SAFETY: We're closing the handle after we're done with it.
    unsafe {
        let _ = CloseHandle(handle);
    };
    Ok(buffer)
}