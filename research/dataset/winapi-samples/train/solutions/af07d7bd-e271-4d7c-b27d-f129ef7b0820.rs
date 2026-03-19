use windows::core::{Error, Result};
use windows::Win32::Foundation::E_FAIL;
use windows::Win32::UI::Input::Ime::{ImmGetContext, ImmGetOpenStatus, ImmReleaseContext};
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

fn is_ime_open_for_foreground() -> Result<bool> {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.is_invalid() {
        return Err(Error::from_hresult(E_FAIL));
    }

    let himc = unsafe { ImmGetContext(hwnd) };
    if himc.is_invalid() {
        return Ok(false);
    }

    let is_open = unsafe { ImmGetOpenStatus(himc) }.as_bool();

    let _ = unsafe { ImmReleaseContext(hwnd, himc) };

    Ok(is_open)
}

fn main() {
    if let Ok(open) = is_ime_open_for_foreground() {
        println!("IME is open: {}", open);
    }
}
