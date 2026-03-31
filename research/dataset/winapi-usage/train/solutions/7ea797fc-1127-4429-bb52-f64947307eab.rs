use windows::core::{Error, Result};
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::UI::Accessibility::AccNotifyTouchInteraction;

fn call_acc_notify_touch_interaction() -> windows::core::Result<windows::core::Result<()>> {
    let hwnd = HWND::default();
    let point = POINT { x: 0, y: 0 };
    // SAFETY: Passing default/null handles and a valid POINT struct.
    // The API handles these gracefully for notification purposes.
    Ok(unsafe { AccNotifyTouchInteraction(hwnd, hwnd, point) })
}
