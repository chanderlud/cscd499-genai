use windows::core::{Error, Result};
use windows::Win32::Foundation::RECT;
use windows::Win32::UI::WindowsAndMessaging::{AdjustWindowRect, WS_OVERLAPPEDWINDOW};

fn call_adjust_window_rect() -> windows::core::Result<windows::core::Result<()>> {
    let mut rect = RECT {
        left: 0,
        top: 0,
        right: 100,
        bottom: 100,
    };
    // SAFETY: `rect` is a valid mutable pointer to a RECT structure,
    // `WS_OVERLAPPEDWINDOW` is a valid window style, and `false` is a valid boolean for `bmenu`.
    let res = unsafe { AdjustWindowRect(&mut rect, WS_OVERLAPPEDWINDOW, false) };
    Ok(res)
}
