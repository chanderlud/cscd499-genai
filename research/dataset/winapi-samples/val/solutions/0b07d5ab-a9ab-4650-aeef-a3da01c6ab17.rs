use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, ERROR_WRITE_FAULT, HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, ReadFile, WriteFile, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL,
    FILE_FLAGS_AND_ATTRIBUTES, FILE_GENERIC_READ, FILE_GENERIC_WRITE,
};

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

struct FileHandle(HANDLE);

impl Drop for FileHandle {
    fn drop(&mut self) {
        if self.0 != INVALID_HANDLE_VALUE {
            unsafe {
                // SAFETY: We're closing a valid handle that we own
                let _ = CloseHandle(self.0);
            }
        }
    }
}

pub fn copy_file_chunked(src_path: &str, dst_path: &str) -> Result<()> {
    let src_wide = wide_null(OsStr::new(src_path));
    let dst_wide = wide_null(OsStr::new(dst_path));

    // Open source file for reading
    let src_handle = unsafe {
        // SAFETY: We're calling CreateFileW with valid parameters
        CreateFileW(
            PCWSTR(src_wide.as_ptr()),
            FILE_GENERIC_READ.0,
            Default::default(),
            None,
            Default::default(),
            FILE_FLAGS_AND_ATTRIBUTES(0),
            None,
        )?
    };

    let src_file = FileHandle(src_handle);

    // Create destination file for writing
    let dst_handle = unsafe {
        // SAFETY: We're calling CreateFileW with valid parameters
        CreateFileW(
            PCWSTR(dst_wide.as_ptr()),
            FILE_GENERIC_WRITE.0,
            Default::default(),
            None,
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )?
    };

    let dst_file = FileHandle(dst_handle);

    // Copy file contents in chunks
    let mut buffer = [0u8; 4096];

    loop {
        let mut bytes_read = 0u32;

        // Read chunk from source
        unsafe {
            // SAFETY: We're calling ReadFile with valid parameters
            ReadFile(
                src_file.0,
                Some(&mut buffer[..]),
                Some(&mut bytes_read),
                None,
            )?;
        }

        // End of file reached
        if bytes_read == 0 {
            break;
        }

        // Write chunk to destination
        let mut bytes_written = 0u32;
        unsafe {
            // SAFETY: We're calling WriteFile with valid parameters
            WriteFile(
                dst_file.0,
                Some(&buffer[..bytes_read as usize]),
                Some(&mut bytes_written),
                None,
            )?;
        }

        // Verify all bytes were written
        if bytes_written != bytes_read {
            return Err(Error::from_hresult(windows::core::HRESULT::from_win32(
                ERROR_WRITE_FAULT.0,
            )));
        }
    }

    Ok(())
}
