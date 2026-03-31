use windows::core::{w, Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Networking::WinInet::AppCacheCheckManifest;

fn call_app_cache_check_manifest() -> WIN32_ERROR {
    unsafe {
        WIN32_ERROR(AppCacheCheckManifest(
            w!("http://example.com/"),
            w!("http://example.com/manifest.appcache"),
            &[],
            &[],
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        ))
    }
}
