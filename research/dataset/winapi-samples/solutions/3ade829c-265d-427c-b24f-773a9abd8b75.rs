// Dynamically load and use GetDpiForWindow function

use std::iter::once;
use std::os::windows::prelude::OsStrExt;
use windows::core::{Error, Result, PCSTR, PCWSTR};
use windows::Win32::Foundation::{FARPROC, HWND};
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
use windows::Win32::UI::WindowsAndMessaging::GetDesktopWindow;

fn encode_wide(string: impl AsRef<std::ffi::OsStr>) -> Vec<u16> {
    string.as_ref().encode_wide().chain(once(0)).collect()
}

fn get_function_impl(library: &str, function: &str) -> Result<FARPROC> {
    let library_wide = encode_wide(library);
    assert!(
        function.ends_with('\0'),
        "Function name must be null-terminated"
    );

    // SAFETY: LoadLibraryW expects a null-terminated wide string
    let module = unsafe { LoadLibraryW(PCWSTR::from_raw(library_wide.as_ptr())) }?;

    // SAFETY: GetProcAddress expects a null-terminated ASCII string
    let func_ptr = unsafe { GetProcAddress(module, PCSTR::from_raw(function.as_ptr())) };

    Ok(func_ptr)
}

fn main() -> Result<()> {
    // Dynamically load GetDpiForWindow function from user32.dll
    let get_dpi_for_window = get_function_impl("user32.dll", "GetDpiForWindow\0")?;

    if let Some(func) = get_dpi_for_window {
        // SAFETY: Transmuting to the correct function signature
        let get_dpi_for_window: unsafe extern "system" fn(HWND) -> u32 =
            unsafe { std::mem::transmute(func) };

        // SAFETY: GetDesktopWindow is always safe to call
        let desktop = unsafe { GetDesktopWindow() };

        // SAFETY: Calling the dynamically loaded function with valid HWND
        let dpi = unsafe { get_dpi_for_window(desktop) };
        println!("DPI for desktop window: {}", dpi);
    } else {
        println!("GetDpiForWindow not available on this system");
    }

    Ok(())
}
