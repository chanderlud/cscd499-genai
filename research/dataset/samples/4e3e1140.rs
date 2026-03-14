use std::os::raw::c_void;
use windows::core::{Result, BOOL};
use windows::Win32::Foundation::{HWND, LPARAM};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
use windows::Win32::UI::Input::KeyboardAndMouse::IsWindowEnabled;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowLongW, GetWindowTextLengthW, GetWindowTextW, IsWindowVisible,
    GWL_EXSTYLE, WS_EX_APPWINDOW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
};

fn main() -> Result<()> {
    // Enumerate top-level windows
    unsafe {
        let _ = EnumWindows(Some(enum_window_callback), LPARAM(0));
    }
    Ok(())
}

// Window enumeration callback
unsafe extern "system" fn enum_window_callback(hwnd: HWND, _l_param: LPARAM) -> BOOL {
    // Check if window is visible
    if !IsWindowVisible(hwnd).as_bool() {
        return BOOL::from(true);
    }

    // Check if window is enabled
    if !IsWindowEnabled(hwnd).as_bool() {
        return BOOL::from(true);
    }

    // Check if window is cloaked (hidden by DWM)
    let mut cloaked: u32 = 0;
    let hr = DwmGetWindowAttribute(
        hwnd,
        DWMWA_CLOAKED,
        &mut cloaked as *mut _ as *mut c_void,
        std::mem::size_of::<u32>() as u32,
    );
    if hr.is_ok() && cloaked != 0 {
        return BOOL::from(true);
    }

    // Get extended window styles
    let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
    if ex_style & WS_EX_APPWINDOW.0 != 0
        || ex_style & WS_EX_NOACTIVATE.0 != 0
        || ex_style & WS_EX_TOOLWINDOW.0 != 0
    {
        return BOOL::from(true);
    }

    // Get window title length
    let length = GetWindowTextLengthW(hwnd);

    // Only add the window to the window list, if:
    // it has a title
    // and none of above predicates are matching...
    if length > 0 {
        // Create a buffer to hold the window title (length elements, no +1 needed)
        let mut buffer = vec![0u16; length as usize];
        let text_len = GetWindowTextW(hwnd, &mut buffer);
        if text_len > 0 {
            // Convert to UTF-16 string and print
            if let Ok(title_str) = String::from_utf16(&buffer) {
                println!("Visible window: {}", title_str);
            }
        }
    }

    // Continue enumeration
    BOOL::from(true)
}
