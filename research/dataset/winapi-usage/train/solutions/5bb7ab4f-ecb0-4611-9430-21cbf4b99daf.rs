use windows::core::{Error, Result};
use windows::Win32::Foundation::{ERROR_SUCCESS, HWND, WIN32_ERROR};
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::UI::Accessibility::AccessibleObjectFromEvent;
use windows::Win32::UI::Accessibility::IAccessible;

fn call_accessible_object_from_event() -> WIN32_ERROR {
    let hwnd = HWND::default();
    let dwid: u32 = 0;
    let dwchildid: u32 = 0;
    let ppacc: *mut Option<IAccessible> = std::ptr::null_mut();
    let pvarchild: *mut VARIANT = std::ptr::null_mut();

    unsafe {
        match AccessibleObjectFromEvent(hwnd, dwid, dwchildid, ppacc, pvarchild) {
            Ok(_) => ERROR_SUCCESS,
            Err(e) => WIN32_ERROR::from_error(&e).unwrap_or(ERROR_SUCCESS),
        }
    }
}
