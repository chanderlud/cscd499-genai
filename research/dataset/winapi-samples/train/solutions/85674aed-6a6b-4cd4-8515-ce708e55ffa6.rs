use windows::core::{Error, Result, HRESULT, PCSTR};
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};
use windows::Win32::UI::Input::Ime::ImmGetDefaultIMEWnd;
use windows::Win32::UI::WindowsAndMessaging::{SendMessageW, WM_IME_CONTROL};

const IMC_GETCONVERSIONMODE: u32 = 0x0001;

fn get_ime_conversion_mode_dynamic() -> Result<i32> {
    // Load user32.dll dynamically
    let user32_module = unsafe { LoadLibraryA(PCSTR(b"user32.dll\0".as_ptr())) }?;

    // Get GetForegroundWindow by ordinal 6801
    let proc_address = unsafe { GetProcAddress(user32_module, PCSTR(6801 as usize as *const u8)) };

    if proc_address.is_none() {
        return Err(Error::from_thread());
    }

    // Cast to function pointer type
    type GetForegroundWindowFn = unsafe extern "system" fn() -> HWND;
    let get_foreground_window: GetForegroundWindowFn =
        unsafe { std::mem::transmute(proc_address.unwrap()) };

    // Get foreground window handle
    let foreground_hwnd = unsafe { get_foreground_window() };
    if foreground_hwnd.is_invalid() {
        return Err(Error::from_thread());
    }

    // Get default IME window for foreground window
    let ime_hwnd = unsafe { ImmGetDefaultIMEWnd(foreground_hwnd) };
    if ime_hwnd.is_invalid() {
        return Err(Error::from_thread());
    }

    // Query conversion mode via SendMessageW
    let result = unsafe {
        SendMessageW(
            ime_hwnd,
            WM_IME_CONTROL,
            Some(WPARAM(IMC_GETCONVERSIONMODE as usize)),
            Some(LPARAM(0)),
        )
    };

    Ok(result.0 as i32)
}
