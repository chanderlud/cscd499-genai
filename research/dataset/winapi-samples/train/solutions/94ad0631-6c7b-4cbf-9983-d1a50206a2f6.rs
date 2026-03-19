use windows::core::{Error, Result};
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::Input::Ime::ImmGetDefaultIMEWnd;
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, SendMessageW, WM_IME_CONTROL};

// Define the constant since it's not in the windows crate
const IMC_GETCONVERSIONMODE: u32 = 0x0001;

fn get_ime_conversion_mode() -> Result<i32> {
    // Get the foreground window handle
    let hwnd_foreground = unsafe { GetForegroundWindow() };
    if hwnd_foreground.is_invalid() {
        return Err(Error::from_thread());
    }

    // Get the default IME window handle for the foreground window
    let hwnd_ime = unsafe { ImmGetDefaultIMEWnd(hwnd_foreground) };
    if hwnd_ime.is_invalid() {
        return Err(Error::from_thread());
    }

    // Send message to query the current conversion mode
    let result = unsafe {
        SendMessageW(
            hwnd_ime,
            WM_IME_CONTROL,
            Some(WPARAM(IMC_GETCONVERSIONMODE as usize)),
            Some(LPARAM(0)),
        )
    };

    // SendMessageW returns LRESULT, which we convert to i32
    Ok(result.0 as i32)
}

fn main() -> Result<()> {
    let conversion_mode = get_ime_conversion_mode()?;
    println!("IME Conversion Mode: {}", conversion_mode);
    Ok(())
}
