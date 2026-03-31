use windows::core::Error;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::UI::Magnification::{MagGetFullscreenColorEffect, MAGCOLOREFFECT};

fn call_mag_get_fullscreen_color_effect() -> WIN32_ERROR {
    let mut effect = MAGCOLOREFFECT::default();
    // SAFETY: `effect` is a valid mutable pointer to a MAGCOLOREFFECT struct.
    let success = unsafe { MagGetFullscreenColorEffect(&mut effect) };
    if success.as_bool() {
        WIN32_ERROR(0)
    } else {
        let err = Error::from_thread();
        WIN32_ERROR::from_error(&err).unwrap_or_else(|| WIN32_ERROR(1))
    }
}
