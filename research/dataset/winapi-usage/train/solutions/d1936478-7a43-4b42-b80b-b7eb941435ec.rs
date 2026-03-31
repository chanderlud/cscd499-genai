use windows::core::{Error, Result};
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Dwm::{DwmEnableBlurBehindWindow, DWM_BB_ENABLE, DWM_BLURBEHIND};

fn call_dwm_enable_blur_behind_window() -> Result<Result<()>> {
    let blur_behind = DWM_BLURBEHIND {
        dwFlags: DWM_BB_ENABLE,
        fEnable: true.into(),
        hRgnBlur: Default::default(),
        fTransitionOnMaximized: false.into(),
    };
    // SAFETY: DwmEnableBlurBehindWindow requires a valid pointer to DWM_BLURBEHIND.
    // We pass a reference to a locally initialized struct, which is valid for the call.
    Ok(unsafe { DwmEnableBlurBehindWindow(HWND::default(), &blur_behind) })
}
