use windows::core::{Error, Result};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{CallWindowProcW, WNDPROC};

fn call_call_window_proc_w() -> Result<LRESULT> {
    // Create a dummy WNDPROC - using None as a safe default
    let wndproc: WNDPROC = None;

    // Create a dummy HWND
    let hwnd = HWND::default();

    // Use WM_NULL (0x0000) as a common message
    let msg = 0x0000u32;

    // Use default WPARAM and LPARAM values
    let wparam = WPARAM(0);
    let lparam = LPARAM(0);

    // Call the API in an unsafe block
    // CallWindowProcW returns LRESULT directly, not a Result
    // We wrap it in Ok since the function doesn't return error codes
    let result = unsafe { CallWindowProcW(wndproc, hwnd, msg, wparam, lparam) };

    Ok(result)
}
