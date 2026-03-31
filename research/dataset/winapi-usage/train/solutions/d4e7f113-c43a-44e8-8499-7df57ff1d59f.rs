#![deny(warnings)]

use windows::core::HRESULT;
use windows::Win32::Graphics::Gdi::{Arc, HDC};

#[allow(dead_code)]
fn call_arc() -> HRESULT {
    let hdc = HDC(std::ptr::null_mut());
    let success = unsafe { Arc(hdc, 0, 0, 100, 100, 0, 0, 100, 100) };
    if success.as_bool() {
        HRESULT(0)
    } else {
        HRESULT::from_win32(1u32)
    }
}
