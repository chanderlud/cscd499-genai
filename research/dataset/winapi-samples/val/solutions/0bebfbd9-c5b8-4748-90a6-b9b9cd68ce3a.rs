use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::Ime::{
    ImmGetCompositionStringW, ImmGetContext, ImmReleaseContext, GCS_COMPSTR, HIMC,
};
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

fn get_foreground_ime_composition_string() -> Result<String> {
    // Get the foreground window handle
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.is_invalid() {
        // No foreground window, return empty string
        return Ok(String::new());
    }

    // Get the IME context for the window
    let himc = unsafe { ImmGetContext(hwnd) };
    if himc.is_invalid() {
        // No IME context available, return empty string
        return Ok(String::new());
    }

    // Ensure we release the context when we're done
    struct ContextGuard(HWND, HIMC);
    impl Drop for ContextGuard {
        fn drop(&mut self) {
            unsafe { ImmReleaseContext(self.0, self.1) };
        }
    }
    let _guard = ContextGuard(hwnd, himc);

    // First call to get required buffer size
    let size = unsafe { ImmGetCompositionStringW(himc, GCS_COMPSTR, None, 0) };

    if size == 0 {
        // No composition string active
        return Ok(String::new());
    }

    if size < 0 {
        // Error occurred
        return Err(Error::from_hresult(HRESULT::from_win32(-size as u32)));
    }

    // Allocate buffer for the composition string
    let mut buffer = vec![0u16; (size as usize) / 2];

    // Get the actual composition string
    let result = unsafe {
        ImmGetCompositionStringW(
            himc,
            GCS_COMPSTR,
            Some(buffer.as_mut_ptr() as *mut _),
            size as u32,
        )
    };

    if result < 0 {
        return Err(Error::from_hresult(HRESULT::from_win32(-result as u32)));
    }

    // Convert UTF-16 buffer to Rust String
    let composition = String::from_utf16_lossy(&buffer);
    Ok(composition)
}
