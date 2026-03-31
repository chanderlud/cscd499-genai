use windows::core::{Error, Result};
use windows::Win32::Foundation::{GetLastError, WIN32_ERROR};

fn call_get_last_error() -> WIN32_ERROR {
    // SAFETY: GetLastError is a standard Win32 API that safely retrieves the calling thread's last-error code.
    unsafe { GetLastError() }
}
