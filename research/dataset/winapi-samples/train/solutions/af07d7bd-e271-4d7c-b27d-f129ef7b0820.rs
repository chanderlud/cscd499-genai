use windows::core::{Error, Result};
use windows::Win32::Foundation::E_FAIL;
use windows::Win32::UI::Input::Ime::{ImmGetContext, ImmGetOpenStatus, ImmReleaseContext};
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

fn is_ime_open_for_foreground() -> Result<bool> {
    // Get the foreground window handle
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.is_invalid() {
        // No foreground window available
        return Err(Error::from_hresult(E_FAIL));
    }

    // Get the IME context for the window
    let himc = unsafe { ImmGetContext(hwnd) };
    if himc.is_invalid() {
        // No IME context available for this window
        return Ok(false);
    }

    // Check the IME open status
    let is_open = unsafe { ImmGetOpenStatus(himc) }.as_bool();

    // Release the IME context
    unsafe { ImmReleaseContext(hwnd, himc) };

    Ok(is_open)
}
