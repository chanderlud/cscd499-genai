use std::path::Path;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows::core::{Result, Error, PCWSTR};
use windows::Win32::Foundation::{HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT, INVALID_HANDLE_VALUE, CloseHandle};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, ReadDirectoryChangesW, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OVERLAPPED,
    FILE_LIST_DIRECTORY, FILE_SHARE_READ, FILE_SHARE_WRITE, FILE_SHARE_DELETE,
    OPEN_EXISTING, FILE_NOTIFY_INFORMATION, FILE_ACTION_ADDED, FILE_NOTIFY_CHANGE_FILE_NAME,
};
use windows::Win32::System::IO::{OVERLAPPED, GetOverlappedResult};
use windows::Win32::System::Threading::{CreateEventW, WaitForSingleObject};

fn wide_null(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt};
    s.encode_wide().chain(once(0)).collect()
}

pub fn wait_for_create(dir: &Path, timeout_ms: u32) -> Result<Option<OsString>> {
    // Convert directory path to wide string
    let dir_wide = wide_null(dir.as_os_str());
    
    // Open directory handle with required flags
    let dir_handle = unsafe {
        CreateFileW(
            PCWSTR(dir_wide.as_ptr()),
            FILE_LIST_DIRECTORY.0,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            None,
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OVERLAPPED,
            None,
        )?
    };
    
    // Create event for overlapped I/O
    let event = unsafe { CreateEventW(None, true, false, None)? };
    
    // Set up overlapped structure
    let mut overlapped = OVERLAPPED::default();
    overlapped.hEvent = event;
    
    // Buffer for directory change notifications
    let mut buffer = [0u8; 1024];
    let mut bytes_returned = 0u32;
    
    // Start monitoring directory
    unsafe {
        ReadDirectoryChangesW(
            dir_handle,
            buffer.as_mut_ptr() as *mut _,
            buffer.len() as u32,
            false, // Don't watch subtree
            FILE_NOTIFY_CHANGE_FILE_NAME, // Watch for file name changes
            Some(&mut bytes_returned),
            Some(&mut overlapped),
            None,
        )?;
    }
    
    // Wait for event with timeout
    let wait_result = unsafe { WaitForSingleObject(event, timeout_ms) };
    
    match wait_result {
        WAIT_OBJECT_0 => {
            // Get the result of the overlapped operation
            let mut bytes_transferred = 0u32;
            unsafe {
                GetOverlappedResult(
                    dir_handle,
                    &overlapped,
                    &mut bytes_transferred,
                    false,
                )?;
            }
            
            // Parse the notification buffer
            if bytes_transferred > 0 {
                let notify = buffer.as_ptr() as *const FILE_NOTIFY_INFORMATION;
                let action = unsafe { (*notify).Action };
                
                if action == FILE_ACTION_ADDED {
                    // Extract the file name
                    let name_length = unsafe { (*notify).FileNameLength } as usize;
                    let name_ptr = unsafe { (*notify).FileName.as_ptr() };
                    let name_slice = unsafe { std::slice::from_raw_parts(name_ptr, name_length / 2) };
                    let file_name = OsString::from_wide(name_slice);
                    
                    // Clean up handles
                    unsafe {
                        let _ = CloseHandle(dir_handle);
                        let _ = CloseHandle(event);
                    }
                    
                    return Ok(Some(file_name));
                }
            }
            
            // Clean up handles
            unsafe {
                let _ = CloseHandle(dir_handle);
                let _ = CloseHandle(event);
            }
            
            Ok(None)
        }
        WAIT_TIMEOUT => {
            // Clean up handles
            unsafe {
                let _ = CloseHandle(dir_handle);
                let _ = CloseHandle(event);
            }
            
            Ok(None)
        }
        _ => {
            // Clean up handles
            unsafe {
                let _ = CloseHandle(dir_handle);
                let _ = CloseHandle(event);
            }
            
            Err(Error::from_thread())
        }
    }
}