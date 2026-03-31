#![deny(warnings)]

use windows::core::Result;
use windows::Win32::Networking::WinInet::AppCacheCloseHandle;

#[allow(dead_code)]
fn call_app_cache_close_handle() -> Result<()> {
    // SAFETY: Passing a null handle is safe for this exercise; the API does not return a status code.
    unsafe {
        AppCacheCloseHandle(std::ptr::null());
    }
    Ok(())
}
