use std::path::Path;
use windows::core::{Result, Error, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE, ERROR_FILE_NOT_FOUND};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, WriteFile, FlushFileBuffers, GetTempFileNameW, ReplaceFileW, MoveFileExW,
    FILE_FLAGS_AND_ATTRIBUTES, FILE_GENERIC_WRITE, FILE_SHARE_MODE, CREATE_ALWAYS,
    MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
};
use windows::Win32::System::SystemServices::GENERIC_WRITE;

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

pub fn atomic_write(path: &Path, data: &[u8]) -> Result<()> {
    // Get parent directory for temp file creation
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let parent_w = wide_null(parent.as_os_str());
    
    // Generate temp file name in same directory
    let mut temp_name = [0u16; 260]; // MAX_PATH
    let prefix = wide_null("tmp".as_ref());
    
    // SAFETY: GetTempFileNameW writes to temp_name buffer
    let result = unsafe {
        GetTempFileNameW(
            PCWSTR(parent_w.as_ptr()),
            PCWSTR(prefix.as_ptr()),
            0,
            temp_name.as_mut_ptr(),
        )
    };
    
    if result == 0 {
        return Err(Error::from_win32());
    }
    
    // Create temp file
    let temp_path = Path::new(&String::from_utf16_lossy(&temp_name[..temp_name.iter().position(|&x| x == 0).unwrap_or(temp_name.len())]));
    let temp_w = wide_null(temp_path.as_os_str());
    
    // SAFETY: CreateFileW returns a handle that must be closed
    let temp_handle = unsafe {
        CreateFileW(
            PCWSTR(temp_w.as_ptr()),
            GENERIC_WRITE.0,
            FILE_SHARE_MODE(0),
            None,
            CREATE_ALWAYS,
            FILE_FLAGS_AND_ATTRIBUTES(0),
            None,
        )
    }?;
    
    // Ensure handle is closed on error
    let _guard = HandleGuard(temp_handle);
    
    // Write data to temp file
    let mut bytes_written = 0;
    // SAFETY: WriteFile writes data from buffer to file
    unsafe {
        WriteFile(
            temp_handle,
            Some(data.as_ptr() as *const std::ffi::c_void),
            data.len() as u32,
            Some(&mut bytes_written),
            None,
        )?;
    }
    
    // Flush to ensure data is written to disk
    // SAFETY: FlushFileBuffers flushes the file buffer
    unsafe {
        FlushFileBuffers(temp_handle)?;
    }
    
    // Close handle before replacement (guard will do this, but we need it closed now)
    drop(_guard);
    
    // Try atomic replace with ReplaceFileW
    let dest_w = wide_null(path.as_os_str());
    let backup_w = wide_null("backup".as_ref()); // Not used, but required parameter
    
    // SAFETY: ReplaceFileW atomically replaces the destination file
    let replace_result = unsafe {
        ReplaceFileW(
            PCWSTR(dest_w.as_ptr()),
            PCWSTR(temp_w.as_ptr()),
            PCWSTR(backup_w.as_ptr()),
            0,
            None,
            None,
        )
    };
    
    match replace_result {
        Ok(()) => Ok(()),
        Err(e) => {
            // If destination doesn't exist, use MoveFileExW as fallback
            if e.code() == HRESULT::from_win32(ERROR_FILE_NOT_FOUND.0) {
                // SAFETY: MoveFileExW moves the temp file to destination
                unsafe {
                    MoveFileExW(
                        PCWSTR(temp_w.as_ptr()),
                        PCWSTR(dest_w.as_ptr()),
                        MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
                    )?;
                }
                Ok(())
            } else {
                Err(e)
            }
        }
    }
}

// Helper struct to ensure handle is closed
struct HandleGuard(HANDLE);

impl Drop for HandleGuard {
    fn drop(&mut self) {
        if self.0 != INVALID_HANDLE_VALUE {
            // SAFETY: We own the handle and must close it
            unsafe {
                let _ = CloseHandle(self.0);
            }
        }
    }
}