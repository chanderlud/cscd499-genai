use windows::core::{Error, Result};
use windows::Win32::Foundation::{E_INVALIDARG, POINT};
use windows::Win32::UI::Input::Ime::{
    ImmGetContext, ImmReleaseContext, ImmSetCompositionWindow, CFS_POINT, COMPOSITIONFORM,
};
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

#[allow(dead_code)]
fn set_ime_composition_position(x: i32, y: i32) -> Result<()> {
    if x < 0 || y < 0 {
        return Err(Error::from_hresult(E_INVALIDARG));
    }

    // Get the foreground window
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.is_invalid() {
        return Err(Error::from_thread());
    }

    // Get IME context for the foreground window
    let himc = unsafe { ImmGetContext(hwnd) };
    if himc.is_invalid() {
        return Err(Error::from_thread());
    }

    // Set up composition form structure
    let comp_form = COMPOSITIONFORM {
        dwStyle: CFS_POINT,
        ptCurrentPos: POINT { x, y },
        rcArea: Default::default(),
    };

    // Set composition window position
    let result = unsafe { ImmSetCompositionWindow(himc, &comp_form) };

    // Release IME context
    unsafe {
        let _ = ImmReleaseContext(hwnd, himc);
    };

    if result.as_bool() {
        Ok(())
    } else {
        Err(Error::from_thread())
    }
}
