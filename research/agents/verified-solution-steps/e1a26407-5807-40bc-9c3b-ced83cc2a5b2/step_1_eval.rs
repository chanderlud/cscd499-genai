use std::path::Path;
use windows::core::{Result, Error, HRESULT};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, WriteFile, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, GENERIC_WRITE,
    FILE_SHARE_NONE, OPEN_EXISTING,
};
use windows::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE, CloseHandle};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

pub fn write_all(path: &Path, data: &[u8]) -> Result<()> {
    let wide_path = wide_null(path.as_os_str());
    
    // SAFETY: CreateFileW is called with valid parameters
    let handle = unsafe {
        CreateFileW(
            wide_path.as_ptr(),
            GENERIC_WRITE.0,
            FILE_SHARE_NONE,
            std::ptr::null(),
            CREATE_ALWAYS,
            FILE_ATTRIBUTE_NORMAL,
            HANDLE::default(),
        )
    };
    
    if handle == INVALID_HANDLE_VALUE {
        return Err(Error::from_thread());
    }
    
    // Ensure handle is closed even if write fails
    let result = write_all_bytes(handle, data);
    
    // SAFETY: handle is valid and we're done with it
    unsafe {
        let _ = CloseHandle(handle);
    }
    
    result
}

fn write_all_bytes(handle: HANDLE, data: &[u8]) -> Result<()> {
    let mut bytes_written = 0u32;
    let mut remaining = data;
    
    while !remaining.is_empty() {
        // SAFETY: WriteFile is called with valid parameters
        let success = unsafe {
            WriteFile(
                handle,
                remaining.as_ptr(),
                remaining.len() as u32,
                &mut bytes_written,
                std::ptr::null_mut(),
            )
        };
        
        if !success.as_bool() {
            return Err(Error::from_thread());
        }
        
        if bytes_written == 0 {
            // This shouldn't happen with valid file handle, but handle it
            return Err(Error::from_hresult(HRESULT::from_win32(0x8007000C))); // ERROR_WRITE_FAULT
        }
        
        remaining = &remaining[bytes_written as usize..];
    }
    
    Ok(())
}