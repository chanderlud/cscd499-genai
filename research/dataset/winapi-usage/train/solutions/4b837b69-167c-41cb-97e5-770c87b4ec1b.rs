#![deny(warnings)]

use windows::Win32::Foundation::{RECT, WIN32_ERROR};
use windows::Win32::UI::HiDpi::AdjustWindowRectExForDpi;
use windows::Win32::UI::WindowsAndMessaging::{WINDOW_EX_STYLE, WINDOW_STYLE};

#[allow(dead_code)]
fn call_adjust_window_rect_ex_for_dpi() -> WIN32_ERROR {
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 100,
        bottom: 100,
    };
    let style = WINDOW_STYLE(0x00CF0000);
    let ex_style = WINDOW_EX_STYLE(0);
    let dpi = 96u32;

    match unsafe { AdjustWindowRectExForDpi(&mut rect, style, false, ex_style, dpi) } {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
