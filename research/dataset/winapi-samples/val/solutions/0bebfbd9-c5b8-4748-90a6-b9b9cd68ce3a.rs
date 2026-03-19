use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::Ime::{
    ImmGetCompositionStringW, ImmGetContext, ImmReleaseContext, GCS_COMPSTR, HIMC,
};
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

fn get_foreground_ime_composition_string() -> Result<String> {
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.is_invalid() {
        return Ok(String::new());
    }

    let himc = unsafe { ImmGetContext(hwnd) };
    if himc.is_invalid() {
        return Ok(String::new());
    }

    struct ContextGuard(HWND, HIMC);
    impl Drop for ContextGuard {
        fn drop(&mut self) {
            let _ = unsafe { ImmReleaseContext(self.0, self.1) };
        }
    }
    let _guard = ContextGuard(hwnd, himc);

    let size = unsafe { ImmGetCompositionStringW(himc, GCS_COMPSTR, None, 0) };

    if size == 0 {
        return Ok(String::new());
    }

    if size < 0 {
        return Err(Error::from_hresult(HRESULT::from_win32(-size as u32)));
    }

    let mut buffer = vec![0u16; (size as usize) / 2];

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

    let composition = String::from_utf16_lossy(&buffer);
    Ok(composition)
}

fn main() -> Result<()> {
    let composition = get_foreground_ime_composition_string()?;
    println!("Composition: {}", composition);
    Ok(())
}
