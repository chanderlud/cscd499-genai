use windows::core::{Error, Result};
use windows::Win32::Foundation::{RECT, WIN32_ERROR};
use windows::Win32::UI::WindowsAndMessaging::{AdjustWindowRectEx, WINDOW_EX_STYLE, WINDOW_STYLE};

fn call_adjust_window_rect_ex() -> WIN32_ERROR {
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 100,
        bottom: 100,
    };
    let style = WINDOW_STYLE(0x00CF0000);
    let exstyle = WINDOW_EX_STYLE(0);
    let bmenu = false;

    match unsafe { AdjustWindowRectEx(&mut rect, style, bmenu, exstyle) } {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
