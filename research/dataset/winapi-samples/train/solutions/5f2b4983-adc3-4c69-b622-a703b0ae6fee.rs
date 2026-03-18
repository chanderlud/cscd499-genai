use windows::core::{Error, Result};
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::Input::Ime::{ImmGetDefaultIMEWnd, IMC_SETOPENSTATUS};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, SendMessageW, WM_IME_CONTROL};

// Define IMC_GETOPENSTATUS since it's missing from the windows crate
const IMC_GETOPENSTATUS: u32 = 0x0005;

pub fn toggle_ime_open_status() -> Result<bool> {
    // Get the foreground window handle
    let foreground_hwnd = unsafe { GetForegroundWindow() };
    if foreground_hwnd.is_invalid() {
        return Err(Error::from_thread());
    }

    // Get the default IME window for the foreground window
    let ime_hwnd = unsafe { ImmGetDefaultIMEWnd(foreground_hwnd) };
    if ime_hwnd.is_invalid() {
        return Err(Error::from_thread());
    }

    // Query current IME open status
    let current_status = unsafe {
        SendMessageW(
            ime_hwnd,
            WM_IME_CONTROL,
            Some(WPARAM(IMC_GETOPENSTATUS as usize)),
            Some(LPARAM(0)),
        )
    };

    // Convert LRESULT to bool (0 = closed, non-zero = open)
    let is_open = current_status.0 != 0;
    let new_status = !is_open;

    // Set the new open status
    let set_result = unsafe {
        SendMessageW(
            ime_hwnd,
            WM_IME_CONTROL,
            Some(WPARAM(IMC_SETOPENSTATUS as usize)),
            Some(LPARAM(new_status as isize)),
        )
    };

    // Check if the set operation succeeded (non-zero return indicates success)
    if set_result.0 == 0 {
        return Err(Error::from_thread());
    }

    Ok(new_status)
}
