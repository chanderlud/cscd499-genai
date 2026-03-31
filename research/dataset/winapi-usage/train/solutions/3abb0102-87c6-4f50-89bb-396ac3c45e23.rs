#![deny(warnings)]

use windows::core::{Error, Result, HRESULT};
use windows::Win32::Networking::WinInet::{AppCacheCheckManifest, APP_CACHE_STATE};

#[allow(dead_code)]
fn call_app_cache_check_manifest() -> Result<u32> {
    let mut state = APP_CACHE_STATE(0);
    let mut handle: *mut core::ffi::c_void = core::ptr::null_mut();

    // SAFETY: AppCacheCheckManifest is called with valid pointers and string literals.
    let result = unsafe {
        AppCacheCheckManifest(
            windows::core::w!("http://example.com/"),
            windows::core::w!("http://example.com/manifest.appcache"),
            &[],
            &[],
            &mut state,
            &mut handle,
        )
    };

    if result == 0 {
        Ok(result)
    } else {
        Err(Error::from_hresult(HRESULT::from_win32(result)))
    }
}
