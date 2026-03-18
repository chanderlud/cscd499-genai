use windows::core::{Error, Result};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT};
use windows::Win32::UI::Input::Ime::{
    ImmGetCompositionStringW, ImmGetContext, ImmReleaseContext, GCS_COMPATTR, HIMC,
};
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

fn get_composition_attributes() -> Result<Vec<u8>> {
    // Get the foreground window handle
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.is_invalid() {
        return Err(Error::from_thread());
    }

    // Get the IME context for the window
    let himc = unsafe { ImmGetContext(hwnd) };
    if himc.is_invalid() {
        // No IME context available - return empty vector
        return Ok(Vec::new());
    }

    // Ensure we release the context when done
    struct ContextGuard(HWND, HIMC);
    impl Drop for ContextGuard {
        fn drop(&mut self) {
            unsafe { ImmReleaseContext(self.0, self.1) };
        }
    }
    let _guard = ContextGuard(hwnd, himc);

    // First call to get required buffer size
    let size = unsafe { ImmGetCompositionStringW(himc, GCS_COMPATTR, None, 0) };

    // Check for errors or no composition string
    if size < 0 {
        // Convert error code to HRESULT and then to Error
        let hr = windows::core::HRESULT::from_win32(size as u32);
        return Err(windows::core::Error::from_hresult(hr));
    }

    if size == 0 {
        // No composition string active
        return Ok(Vec::new());
    }

    // Allocate buffer and retrieve attributes
    let mut buffer = vec![0u8; size as usize];
    let result = unsafe {
        ImmGetCompositionStringW(
            himc,
            GCS_COMPATTR,
            Some(buffer.as_mut_ptr() as *mut _),
            buffer.len() as u32,
        )
    };

    if result < 0 {
        let hr = windows::core::HRESULT::from_win32(result as u32);
        return Err(windows::core::Error::from_hresult(hr));
    }

    // Truncate to actual size returned (might be smaller than allocated)
    buffer.truncate(result as usize);
    Ok(buffer)
}
