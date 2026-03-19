use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::Ime::ImmGetDefaultIMEWnd;
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, SendMessageA, WM_IME_CONTROL};

pub fn set_conversion_mode(mode: u32) -> Result<()> {
    // Get the foreground window handle
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.is_invalid() {
        return Err(Error::new(
            HRESULT::from_win32(0),
            "No foreground window available",
        ));
    }

    // Get the default IME window handle for the foreground window
    let ime_hwnd = unsafe { ImmGetDefaultIMEWnd(hwnd) };
    if ime_hwnd.is_invalid() {
        return Err(Error::new(
            HRESULT::from_win32(0),
            "No IME window associated with foreground window",
        ));
    }

    // Send WM_IME_CONTROL message with IMC_SETCONVERSIONMODE
    // The conversion mode is passed in the low-order word of wParam
    let wparam = WPARAM(mode as usize);
    let lparam = LPARAM(0);

    // SAFETY: We have valid window handles and are sending a standard IME control message
    let result = unsafe { SendMessageA(ime_hwnd, WM_IME_CONTROL, wparam, lparam) };

    // Check if SendMessageA indicated failure (though it typically doesn't for this message)
    if result == LRESULT(-1) {
        return Err(Error::from_thread());
    }

    Ok(())
}
