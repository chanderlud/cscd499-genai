use windows::core::{Result, Error};
use windows::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE, WAIT_TIMEOUT};
use windows::Win32::System::IO::{
    CreateIoCompletionPort, GetQueuedCompletionStatus, PostQueuedCompletionStatus,
};
use std::ptr;

pub fn iocp_post_and_drain(keys: &[usize], timeout_ms: u32) -> Result<Vec<usize>> {
    // Create IOCP without associating any file handle
    let iocp = unsafe {
        CreateIoCompletionPort(INVALID_HANDLE_VALUE, None, 0, 0)
    };
    
    if iocp == INVALID_HANDLE_VALUE {
        return Err(Error::from_thread());
    }
    
    // Ensure we close the handle on any exit path
    let _guard = scopeguard::guard(iocp, |handle| {
        unsafe { windows::Win32::Foundation::CloseHandle(handle); }
    });
    
    // Post all completions
    for &key in keys {
        let result = unsafe {
            PostQueuedCompletionStatus(iocp, 0, key, ptr::null_mut())
        };
        
        if !result.as_bool() {
            return Err(Error::from_thread());
        }
    }
    
    // Drain completions
    let mut seen = Vec::with_capacity(keys.len());
    let mut bytes_transferred = 0u32;
    let mut completion_key = 0usize;
    let mut overlapped = ptr::null_mut();
    
    for _ in 0..keys.len() {
        let result = unsafe {
            GetQueuedCompletionStatus(
                iocp,
                &mut bytes_transferred,
                &mut completion_key,
                &mut overlapped,
                timeout_ms,
            )
        };
        
        if !result.as_bool() {
            let err = Error::from_thread();
            // Check if this is a timeout (expected when no more completions)
            if err.code() == WAIT_TIMEOUT.to_hresult() {
                break;
            }
            return Err(err);
        }
        
        seen.push(completion_key);
    }
    
    Ok(seen)
}