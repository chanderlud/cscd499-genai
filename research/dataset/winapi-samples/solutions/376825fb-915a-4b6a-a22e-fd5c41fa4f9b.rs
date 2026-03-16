// Get Window Long Example

use windows::core::{Error, Result};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrW, GetWindowLongW, GWL_STYLE, WINDOW_LONG_PTR_INDEX,
};

fn get_window_long(hwnd: HWND, nindex: WINDOW_LONG_PTR_INDEX) -> Result<isize> {
    // SAFETY: FFI call to Win32 API with valid parameters
    let result = unsafe {
        #[cfg(target_pointer_width = "64")]
        {
            GetWindowLongPtrW(hwnd, nindex)
        }
        #[cfg(target_pointer_width = "32")]
        {
            GetWindowLongW(hwnd, nindex) as isize
        }
    };

    if result == 0 {
        // Check if this is an error or a legitimate zero value
        let error = Error::from_thread();
        if error.code().is_err() {
            return Err(error);
        }
    }

    Ok(result)
}

fn main() -> Result<()> {
    // Get desktop window handle for demonstration
    let desktop_hwnd = unsafe { windows::Win32::UI::WindowsAndMessaging::GetDesktopWindow() };

    // Get the window style
    let style = get_window_long(desktop_hwnd, GWL_STYLE)?;
    println!("Desktop window style: 0x{:X}", style);

    Ok(())
}
