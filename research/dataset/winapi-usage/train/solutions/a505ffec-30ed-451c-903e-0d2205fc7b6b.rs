use windows::core::HRESULT;
use windows::Win32::Networking::WinInet::{AppCacheCheckManifest, APP_CACHE_STATE};

fn call_app_cache_check_manifest() -> HRESULT {
    let mut state = APP_CACHE_STATE(0);
    let mut handle: *mut core::ffi::c_void = std::ptr::null_mut();
    unsafe {
        let code = AppCacheCheckManifest(
            windows::core::w!("http://example.com"),
            windows::core::w!("http://example.com/manifest.appcache"),
            &[],
            &[],
            &mut state,
            &mut handle,
        );
        HRESULT::from_win32(code)
    }
}
