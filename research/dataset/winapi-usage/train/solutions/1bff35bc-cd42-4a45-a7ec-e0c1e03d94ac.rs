use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::RECT;
use windows::Win32::UI::WindowsAndMessaging::{AdjustWindowRect, WINDOW_STYLE};

fn call_adjust_window_rect() -> windows::core::HRESULT {
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 100,
        bottom: 100,
    };
    let style = WINDOW_STYLE(0);
    unsafe {
        AdjustWindowRect(&mut rect, style, false)
            .map(|_| HRESULT::default())
            .unwrap_or_else(|e| e.code())
    }
}
