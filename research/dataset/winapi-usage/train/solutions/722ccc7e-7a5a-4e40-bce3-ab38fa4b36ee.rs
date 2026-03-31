use windows::core::{Error, Result};
use windows::Win32::Foundation::RECT;
use windows::Win32::UI::WindowsAndMessaging::{AdjustWindowRectEx, WINDOW_EX_STYLE, WINDOW_STYLE};

fn call_adjust_window_rect_ex() -> windows::core::Result<windows::core::Result<()>> {
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 100,
        bottom: 100,
    };
    // SAFETY: `rect` is a valid mutable reference to a RECT, which safely coerces to the required pointer.
    let result = unsafe {
        AdjustWindowRectEx(
            &mut rect,
            WINDOW_STYLE(0x00CF0000),
            false,
            WINDOW_EX_STYLE(0),
        )
    };
    Ok(result)
}
