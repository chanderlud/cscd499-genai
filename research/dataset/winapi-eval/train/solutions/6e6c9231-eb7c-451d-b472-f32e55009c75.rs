use std::io;
use std::path::Path;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, ERROR_IO_PENDING, WAIT_OBJECT_0};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, WriteFile, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, FILE_FLAG_OVERLAPPED,
    FILE_GENERIC_WRITE,
};
use windows::Win32::System::Threading::{CreateEventW, WaitForSingleObject};
use windows::Win32::System::IO::{GetOverlappedResult, OVERLAPPED};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

pub fn overlapped_write_all(path: &Path, data: &[u8]) -> io::Result<u32> {
    let path_wide = wide_null(path.as_os_str());

    // Create file with overlapped flag - will fail if parent directory doesn't exist
    let file_handle = unsafe {
        CreateFileW(
            PCWSTR(path_wide.as_ptr()),
            FILE_GENERIC_WRITE.0,
            Default::default(),
            None,
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_NORMAL | FILE_FLAG_OVERLAPPED,
            None,
        )
    }?;

    // Create event for overlapped I/O (manual-reset event)
    let event_handle = unsafe { CreateEventW(None, true, false, None) }?;

    // Initialize OVERLAPPED structure
    let mut overlapped = OVERLAPPED {
        hEvent: event_handle,
        ..Default::default()
    };

    // Write data using overlapped I/O
    let mut bytes_written: u32 = 0;
    let write_result = unsafe {
        WriteFile(
            file_handle,
            Some(data),
            Some(&mut bytes_written),
            Some(&mut overlapped),
        )
    };

    // Handle the result
    let result = match write_result {
        Ok(()) => {
            // WriteFile completed synchronously - bytes_written is already set
            Ok(bytes_written)
        }
        Err(e) => {
            // Check if this is ERROR_IO_PENDING
            if e.code() == ERROR_IO_PENDING.into() {
                // Wait for the overlapped operation to complete
                let wait_result = unsafe { WaitForSingleObject(event_handle, 0xFFFFFFFF) };
                if wait_result != WAIT_OBJECT_0 {
                    Err(io::Error::other("WaitForSingleObject failed"))
                } else {
                    // Get the result of the overlapped operation
                    unsafe {
                        GetOverlappedResult(file_handle, &overlapped, &mut bytes_written, false)
                    }?;
                    Ok(bytes_written)
                }
            } else {
                // Other error occurred
                Err(e.into())
            }
        }
    };

    // Clean up handles - ignore errors during cleanup
    unsafe {
        let _ = CloseHandle(file_handle);
        let _ = CloseHandle(event_handle);
    };

    result
}
