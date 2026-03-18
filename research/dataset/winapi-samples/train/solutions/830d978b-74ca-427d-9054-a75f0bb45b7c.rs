use windows::core::{Result, BOOL};
use windows::Win32::Foundation::{HWND, LPARAM};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
use windows::Win32::UI::Input::KeyboardAndMouse::IsWindowEnabled;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowLongW, GetWindowTextLengthW, GetWindowTextW, IsWindowVisible,
    GWL_EXSTYLE, WS_EX_APPWINDOW, WS_EX_NOACTIVATE,
};

unsafe extern "system" fn enum_window_callback(hwnd: HWND, _lparam: LPARAM) -> BOOL {
    // Check visibility and enabled state
    if !IsWindowVisible(hwnd).as_bool() || !IsWindowEnabled(hwnd).as_bool() {
        return BOOL::from(true);
    }

    // Skip cloaked windows (hidden by DWM)
    let mut cloaked = 0u32;
    if DwmGetWindowAttribute(
        hwnd,
        DWMWA_CLOAKED,
        &mut cloaked as *mut _ as *mut _,
        std::mem::size_of::<u32>() as u32,
    )
    .is_ok() && cloaked != 0
    {
        return BOOL::from(true);
    }

    // Get extended window styles safely
    let ex_style = match GetWindowLongW(hwnd, GWL_EXSTYLE) {
        -1 => return BOOL::from(true),
        other => other,
    };
    if (ex_style as u32 & WS_EX_APPWINDOW.0 != 0) || (ex_style as u32 & WS_EX_NOACTIVATE.0 != 0) {
        return BOOL::from(true);
    }

    // Retrieve window title
    let length = match GetWindowTextLengthW(hwnd) {
        -1 => return BOOL::from(true),
        other => other,
    };
    let mut buffer = vec![0u16; (length + 1) as usize];
    if GetWindowTextW(hwnd, &mut buffer) == -1 {
        return BOOL::from(true);
    }
    let title_str = String::from_utf16_lossy(&buffer);

    // Print window information
    println!("Window handle: {:?}, Title: {}", hwnd, title_str);

    BOOL::from(true)
}

fn main() -> Result<()> {
    unsafe { EnumWindows(Some(enum_window_callback), LPARAM(0)) }?;
    Ok(())
}
