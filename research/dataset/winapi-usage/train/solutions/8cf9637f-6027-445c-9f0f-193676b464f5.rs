use windows::core::{Error, Result};
use windows::Win32::Foundation::{HWND, WIN32_ERROR};
use windows::Win32::Graphics::Dwm::DwmDetachMilContent;

fn call_dwm_detach_mil_content() -> windows::Win32::Foundation::WIN32_ERROR {
    unsafe {
        DwmDetachMilContent(HWND::default())
            .map(|_| WIN32_ERROR(0))
            .unwrap_or_else(|e| WIN32_ERROR(e.code().0 as u32))
    }
}
