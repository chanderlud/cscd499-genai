use windows::core::w;
use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Networking::WinInet::AppCacheDeleteGroup;

fn call_app_cache_delete_group() -> WIN32_ERROR {
    let code = unsafe { AppCacheDeleteGroup(w!("http://example.com/manifest.xml")) };
    WIN32_ERROR(code)
}
