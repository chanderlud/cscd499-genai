use windows::core::{Error, Result};
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::Ime::{ImmGetDefaultIMEWnd, IMC_SETSENTENCEMODE};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, SendMessageW, WM_IME_CONTROL};

pub fn set_ime_sentence_mode(sentence_mode: isize) -> Result<()> {
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

    // Send WM_IME_CONTROL message to set the sentence mode
    let result = unsafe {
        SendMessageW(
            ime_hwnd,
            WM_IME_CONTROL,
            Some(WPARAM(IMC_SETSENTENCEMODE as usize)),
            Some(LPARAM(sentence_mode)),
        )
    };

    // Check if the message was processed successfully
    if result == LRESULT(0) {
        Err(Error::from_thread())
    } else {
        Ok(())
    }
}
