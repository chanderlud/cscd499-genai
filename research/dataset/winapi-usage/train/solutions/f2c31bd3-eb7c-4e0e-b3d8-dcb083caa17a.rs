use windows::core::{Error, Result};
use windows::Win32::Foundation::{HWND, POINT, WIN32_ERROR};
use windows::Win32::UI::Accessibility::AccNotifyTouchInteraction;

fn call_acc_notify_touch_interaction() -> WIN32_ERROR {
    match unsafe { AccNotifyTouchInteraction(HWND::default(), HWND::default(), POINT::default()) } {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
