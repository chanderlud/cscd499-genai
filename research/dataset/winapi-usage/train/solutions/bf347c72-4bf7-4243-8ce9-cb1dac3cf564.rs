use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::RECT;
use windows::Win32::UI::HiDpi::AdjustWindowRectExForDpi;
use windows::Win32::UI::WindowsAndMessaging::{WINDOW_EX_STYLE, WINDOW_STYLE};

fn call_adjust_window_rect_ex_for_dpi() -> HRESULT {
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 100,
        bottom: 100,
    };
    let style = WINDOW_STYLE(0);
    let exstyle = WINDOW_EX_STYLE(0);
    let dpi = 96u32;

    // SAFETY: `rect` is a valid mutable pointer to a RECT struct.
    match unsafe { AdjustWindowRectExForDpi(&mut rect, style, false, exstyle, dpi) } {
        Ok(()) => HRESULT(0),
        Err(e) => e.code(),
    }
}
