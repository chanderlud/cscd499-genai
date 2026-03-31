#![deny(warnings)]

use windows::core::HRESULT;
use windows::Win32::System::Registry::{RegCloseKey, HKEY};

#[allow(dead_code)]
fn call_reg_close_key() -> HRESULT {
    // SAFETY: Passing a null HKEY is safe for demonstration;
    // RegCloseKey will return a WIN32_ERROR which we convert to HRESULT.
    let win32_err = unsafe { RegCloseKey(HKEY(std::ptr::null_mut())) };
    win32_err.to_hresult()
}
