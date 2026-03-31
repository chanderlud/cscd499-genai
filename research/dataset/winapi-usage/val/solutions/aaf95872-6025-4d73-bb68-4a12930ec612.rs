use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Networking::WinInet::AppCacheCloseHandle;

fn call_app_cache_close_handle() -> WIN32_ERROR {
    unsafe {
        AppCacheCloseHandle(std::ptr::null());
    }
    WIN32_ERROR(0)
}
