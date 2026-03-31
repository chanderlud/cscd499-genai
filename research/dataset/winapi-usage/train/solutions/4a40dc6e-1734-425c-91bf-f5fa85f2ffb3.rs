use windows::core::{Error, Result};
use windows::Win32::Foundation::{GetLastError, WIN32_ERROR};

fn call_get_last_error() -> Result<WIN32_ERROR> {
    // SAFETY: GetLastError is safe to call; it only reads thread-local error state.
    Ok(unsafe { GetLastError() })
}
