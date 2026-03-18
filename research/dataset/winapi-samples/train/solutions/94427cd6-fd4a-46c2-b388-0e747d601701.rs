use windows::core::{Result, BOOL};
use windows::Win32::Foundation::{HWND, LPARAM};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowLongW, GetWindowTextLengthW, GetWindowTextW, IsWindowVisible,
    GWL_EXSTYLE, GWL_STYLE, WS_DISABLED, WS_EX_APPWINDOW, WS_EX_NOACTIVATE,
};

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    // SAFETY: The pointer is valid for the duration of the callback
    let windows = unsafe { &mut *(lparam.0 as *mut Vec<(HWND, String)>) };

    // Check if window is visible
    if !IsWindowVisible(hwnd).as_bool() {
        return true.into();
    }

    // Check if window is disabled (WS_DISABLED flag)
    let style = unsafe { GetWindowLongW(hwnd, GWL_STYLE) } as u32;
    if style & WS_DISABLED.0 != 0 {
        return true.into();
    }

    // Check extended window styles
    let ex_style = unsafe { GetWindowLongW(hwnd, GWL_EXSTYLE) } as u32;
    if ex_style & WS_EX_APPWINDOW.0 != 0 || ex_style & WS_EX_NOACTIVATE.0 != 0 {
        return true.into();
    }

    // Check if window is cloaked by DWM
    let mut cloaked = BOOL::default();
    let hr = unsafe {
        DwmGetWindowAttribute(
            hwnd,
            DWMWA_CLOAKED,
            &mut cloaked as *mut _ as *mut _,
            std::mem::size_of::<BOOL>() as u32,
        )
    };

    // If DWM call fails or window is cloaked, skip it
    if hr.is_err() || cloaked.as_bool() {
        return true.into();
    }

    // Get window title
    let title_len = unsafe { GetWindowTextLengthW(hwnd) };
    if title_len > 0 {
        let mut buffer = [0u16; 256];
        let len = unsafe { GetWindowTextW(hwnd, &mut buffer) };

        if len > 0 {
            // Convert to String, truncating to 256 characters
            let title = String::from_utf16_lossy(&buffer[..len as usize]);
            windows.push((hwnd, title));
        }
    } else {
        // Window has no title, add with empty string
        windows.push((hwnd, String::new()));
    }

    true.into()
}

pub fn enumerate_windows_sorted() -> Result<Vec<(HWND, String)>> {
    let mut windows = Vec::new();

    // Enumerate all top-level windows
    unsafe {
        EnumWindows(
            Some(enum_windows_proc),
            LPARAM(&mut windows as *mut _ as isize),
        )?;
    }

    // Sort by window handle in ascending order
    windows.sort_by_key(|(hwnd, _): &(HWND, String)| hwnd.0);

    Ok(windows)
}
