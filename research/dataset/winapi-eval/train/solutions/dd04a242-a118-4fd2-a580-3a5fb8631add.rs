use std::ptr;
use windows::core::{Result, HRESULT};
use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE, WAIT_TIMEOUT};
use windows::Win32::System::IO::{
    CreateIoCompletionPort, GetQueuedCompletionStatus, PostQueuedCompletionStatus,
};

struct IocpHandle(HANDLE);

impl Drop for IocpHandle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}

pub fn iocp_post_and_drain(keys: &[usize], timeout_ms: u32) -> Result<Vec<usize>> {
    // Create IOCP - returns Result<HANDLE>
    let iocp = unsafe { CreateIoCompletionPort(INVALID_HANDLE_VALUE, None, 0, 0) }?;

    let _guard = IocpHandle(iocp);

    // Post all completions
    for &key in keys {
        unsafe {
            PostQueuedCompletionStatus(iocp, 0, key, None)?;
        }
    }

    // Drain completions
    let mut seen = Vec::with_capacity(keys.len());
    let mut bytes_transferred = 0u32;
    let mut completion_key = 0usize;
    let mut overlapped = ptr::null_mut();

    for _ in 0..keys.len() {
        match unsafe {
            GetQueuedCompletionStatus(
                iocp,
                &mut bytes_transferred,
                &mut completion_key,
                &mut overlapped,
                timeout_ms,
            )
        } {
            Ok(()) => seen.push(completion_key),
            Err(err) => {
                // Check if this is a timeout error
                if err.code() == HRESULT::from_win32(WAIT_TIMEOUT.0) {
                    break;
                }
                return Err(err);
            }
        }
    }

    Ok(seen)
}
