use windows::core::{Error, Result, HRESULT};
use windows::Win32::Networking::WinInet::AppCacheDeleteGroup;

fn call_app_cache_delete_group() -> HRESULT {
    unsafe {
        let success =
            AppCacheDeleteGroup(windows::core::w!("http://example.com/manifest.appcache"));
        if success == 0 {
            Error::from_thread().code()
        } else {
            HRESULT::default()
        }
    }
}
