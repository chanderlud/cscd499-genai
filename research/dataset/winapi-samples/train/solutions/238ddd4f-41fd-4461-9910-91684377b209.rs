use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::Input::Ime::{ImmGetDefaultIMEWnd, IMC_SETCONVERSIONMODE};
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, SendMessageA, WM_IME_CONTROL};

const IMC_GETCONVERSIONMODE: u32 = 0x0001;

fn toggle_ime_conversion_mode(target_mode: isize) -> Result<isize> {
    // Get the foreground window handle
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.0.is_null() {
        return Err(Error::new(
            HRESULT::from_win32(0), // Generic error
            "No foreground window exists",
        ));
    }

    // Get the default IME window for the foreground window
    let ime_hwnd = unsafe { ImmGetDefaultIMEWnd(hwnd) };
    if ime_hwnd.0.is_null() {
        return Err(Error::from_thread());
    }

    // Get current conversion mode
    let current_mode = unsafe {
        SendMessageA(
            ime_hwnd,
            WM_IME_CONTROL,
            WPARAM(IMC_GETCONVERSIONMODE as usize),
            LPARAM(0),
        )
    };
    let current_mode = current_mode.0 as isize;

    // Determine new mode based on current mode
    let new_mode = if current_mode == 0 { target_mode } else { 0 };

    // Set the new conversion mode
    let result = unsafe {
        SendMessageA(
            ime_hwnd,
            WM_IME_CONTROL,
            WPARAM(IMC_SETCONVERSIONMODE as usize),
            LPARAM(new_mode),
        )
    };

    // Check if the set operation succeeded
    if result.0 == 0 {
        return Err(Error::from_thread());
    }

    Ok(new_mode)
}

fn main() -> Result<()> {
    // Example usage - replace with actual target mode
    let target_mode = 0x0001;
    let result = toggle_ime_conversion_mode(target_mode)?;
    println!("New IME conversion mode: {:#x}", result);
    Ok(())
}
