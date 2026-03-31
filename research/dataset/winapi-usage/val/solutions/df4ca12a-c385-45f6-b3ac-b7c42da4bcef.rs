#![deny(warnings)]

use windows::core::Error;
use windows::core::HRESULT;
use windows::Win32::Foundation::{HWND, S_OK};
use windows::Win32::Graphics::Dwm::DwmAttachMilContent;

#[allow(dead_code)]
fn call_dwm_attach_mil_content() -> HRESULT {
    unsafe { DwmAttachMilContent(HWND::default()) }
        .map(|_| S_OK)
        .unwrap_or_else(|e: Error| e.code())
}
