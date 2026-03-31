#![deny(warnings)]

use windows::core::HRESULT;
#[allow(unused_imports)]
use windows::core::{Error, Result};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Dwm::DwmDetachMilContent;

#[allow(dead_code)]
fn call_dwm_detach_mil_content() -> HRESULT {
    // SAFETY: Passing a null HWND to DwmDetachMilContent as a concrete parameter value.
    unsafe { DwmDetachMilContent(HWND(std::ptr::null_mut())) }
        .map(|_| HRESULT(0))
        .unwrap_or_else(|e| e.code())
}
