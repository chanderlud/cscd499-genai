use windows::core::{Error, Result};
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::Input::Ime::ImmGetDefaultIMEWnd;
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, SendMessageA, WM_IME_CONTROL};

fn get_ime_sentence_mode() -> Result<i32> {
    // Get the foreground window handle
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.is_invalid() {
        // No foreground window available
        return Err(Error::from_thread());
    }

    // Get the IME window handle for the foreground window
    let ime_hwnd = unsafe { ImmGetDefaultIMEWnd(hwnd) };
    if ime_hwnd.is_invalid() {
        // IME window not available
        return Err(Error::from_thread());
    }

    // Send message to query sentence mode
    // IMC_GETSENTENCEMODE constant is 0x0008 according to Windows API documentation
    let result = unsafe {
        SendMessageA(
            ime_hwnd,
            WM_IME_CONTROL,
            WPARAM(0x0008), // IMC_GETSENTENCEMODE
            LPARAM(0),
        )
    };

    // SendMessageA returns LRESULT, convert to i32
    Ok(result.0 as i32)
}
