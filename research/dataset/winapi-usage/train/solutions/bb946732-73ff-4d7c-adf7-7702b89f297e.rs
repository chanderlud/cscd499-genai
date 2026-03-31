#![deny(warnings)]

use windows::core::{w, Error, Result};
use windows::Win32::Networking::WinInet::AppCacheDeleteGroup;

#[allow(dead_code)]
fn call_app_cache_delete_group() -> Result<u32> {
    let result = unsafe { AppCacheDeleteGroup(w!("http://example.com/manifest.appcache")) };
    if result == 0 {
        Err(Error::from_thread())
    } else {
        Ok(result)
    }
}
