use windows::Win32::Foundation::{HWND, WIN32_ERROR};
use windows::Win32::Graphics::Dwm::{DwmEnableBlurBehindWindow, DWM_BLURBEHIND};
use windows::Win32::Graphics::Gdi::HRGN;

fn call_dwm_enable_blur_behind_window() -> windows::Win32::Foundation::WIN32_ERROR {
    let blur_behind = DWM_BLURBEHIND {
        dwFlags: 0x00000001,
        fEnable: true.into(),
        hRgnBlur: HRGN(std::ptr::null_mut()),
        fTransitionOnMaximized: false.into(),
    };

    // SAFETY: Passing a null HWND and a valid DWM_BLURBEHIND struct pointer.
    // The API will return an error for the invalid window handle, which we convert to WIN32_ERROR.
    let result = unsafe { DwmEnableBlurBehindWindow(HWND(std::ptr::null_mut()), &blur_behind) };

    match result {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
