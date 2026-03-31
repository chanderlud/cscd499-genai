use windows::Win32::Foundation::{CloseHandle, HANDLE, WIN32_ERROR};

fn call_close_handle() -> WIN32_ERROR {
    let handle = HANDLE(std::ptr::null_mut());
    // SAFETY: CloseHandle is safe to call with any handle value; it will return an error for invalid handles.
    match unsafe { CloseHandle(handle) } {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
