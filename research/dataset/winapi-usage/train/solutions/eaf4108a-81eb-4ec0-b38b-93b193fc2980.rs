use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::{HWND, POINT};
use windows::Win32::UI::Accessibility::AccNotifyTouchInteraction;

fn call_acc_notify_touch_interaction() -> HRESULT {
    let result = unsafe {
        AccNotifyTouchInteraction(HWND::default(), HWND::default(), POINT { x: 0, y: 0 })
    };
    match result {
        Ok(()) => HRESULT::default(),
        Err(e) => e.code(),
    }
}
