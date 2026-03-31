use windows::Win32::Foundation::{HWND, WIN32_ERROR};
use windows::Win32::Graphics::Dwm::DwmAttachMilContent;

fn call_dwm_attach_mil_content() -> windows::Win32::Foundation::WIN32_ERROR {
    let hwnd = HWND::default();
    match unsafe { DwmAttachMilContent(hwnd) } {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR::from_error(&e).unwrap_or(WIN32_ERROR(0)),
    }
}
