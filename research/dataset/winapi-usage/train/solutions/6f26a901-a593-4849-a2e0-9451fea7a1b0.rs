use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Dwm::{DwmEnableBlurBehindWindow, DWM_BLURBEHIND};

fn call_dwm_enable_blur_behind_window() -> HRESULT {
    let blur_behind = DWM_BLURBEHIND::default();
    // SAFETY: Passing a valid pointer to DWM_BLURBEHIND and a default HWND to the Win32 API.
    unsafe {
        match DwmEnableBlurBehindWindow(HWND::default(), &blur_behind) {
            Ok(()) => HRESULT(0),
            Err(e) => e.code(),
        }
    }
}
