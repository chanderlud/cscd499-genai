use windows::core::HRESULT;
use windows::core::{Error, Result};
use windows::Win32::Foundation::RECT;
use windows::Win32::UI::WindowsAndMessaging::{AdjustWindowRectEx, WINDOW_EX_STYLE, WINDOW_STYLE};

fn call_adjust_window_rect_ex() -> HRESULT {
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 100,
        bottom: 100,
    };
    unsafe {
        AdjustWindowRectEx(&mut rect, WINDOW_STYLE(0), false, WINDOW_EX_STYLE(0))
            .map(|_| HRESULT(0))
            .unwrap_or_else(|e| e.code())
    }
}
