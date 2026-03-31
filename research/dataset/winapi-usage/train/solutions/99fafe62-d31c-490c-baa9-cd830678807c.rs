use windows::core::{Error, Result};
use windows::Win32::Foundation::{HWND, WIN32_ERROR};
use windows::Win32::UI::Magnification::{MagGetColorEffect, MAGCOLOREFFECT};

fn call_mag_get_color_effect() -> WIN32_ERROR {
    let mut effect = MAGCOLOREFFECT::default();
    let hwnd = HWND::default();

    // SAFETY: Passing a valid mutable pointer to MAGCOLOREFFECT and a default HWND.
    // MagGetColorEffect writes to the provided pointer and returns a BOOL indicating success.
    let success = unsafe { MagGetColorEffect(hwnd, &mut effect) };

    if success.as_bool() {
        WIN32_ERROR(0)
    } else {
        let err = Error::from_thread();
        WIN32_ERROR(err.code().0 as u32)
    }
}
