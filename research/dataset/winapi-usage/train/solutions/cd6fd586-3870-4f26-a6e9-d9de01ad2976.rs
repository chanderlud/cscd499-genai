use windows::core::{Error, Result};
use windows::Win32::Foundation::RECT;
use windows::Win32::UI::HiDpi::AdjustWindowRectExForDpi;
use windows::Win32::UI::WindowsAndMessaging::{WINDOW_EX_STYLE, WINDOW_STYLE};

fn call_adjust_window_rect_ex_for_dpi() -> windows::core::Result<windows::core::Result<()>> {
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 800,
        bottom: 600,
    };
    let result = unsafe {
        AdjustWindowRectExForDpi(&mut rect, WINDOW_STYLE(0), false, WINDOW_EX_STYLE(0), 96)
    };
    Ok(result)
}
