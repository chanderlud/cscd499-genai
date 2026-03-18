// TITLE: Redrawing Window Title Bar with DefWindowProcW

use windows::core::Result;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{DefWindowProcW, WM_NCACTIVATE};

fn redraw_title_bar(hwnd: HWND, is_active: bool) -> Result<()> {
    // This pattern toggles the title bar appearance by sending WM_NCACTIVATE messages
    // First deactivate, then reactivate (or vice versa) to force a redraw
    unsafe {
        if is_active {
            // For active window: deactivate then reactivate
            DefWindowProcW(hwnd, WM_NCACTIVATE, WPARAM::default(), LPARAM::default());
            DefWindowProcW(hwnd, WM_NCACTIVATE, WPARAM(1), LPARAM::default());
        } else {
            // For inactive window: activate then deactivate
            DefWindowProcW(hwnd, WM_NCACTIVATE, WPARAM(1), LPARAM::default());
            DefWindowProcW(hwnd, WM_NCACTIVATE, WPARAM::default(), LPARAM::default());
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    // Example usage with a dummy window handle
    // In real code, you would get this from CreateWindowExW or FindWindowW
    let hwnd = HWND(0x12345678 as *mut core::ffi::c_void); // Dummy handle for demonstration

    // Redraw title bar as if window is active
    redraw_title_bar(hwnd, true)?;

    // Redraw title bar as if window is inactive
    redraw_title_bar(hwnd, false)?;

    Ok(())
}
