use windows::core::{Error, Result, HRESULT};
use windows::Win32::UI::Magnification::{MagGetFullscreenColorEffect, MAGCOLOREFFECT};

fn call_mag_get_fullscreen_color_effect() -> HRESULT {
    let mut effect = MAGCOLOREFFECT::default();
    // SAFETY: `effect` is a valid mutable reference to a MAGCOLOREFFECT struct.
    let success = unsafe { MagGetFullscreenColorEffect(&mut effect) };
    if success.0 != 0 {
        HRESULT::from_win32(0)
    } else {
        Error::from_thread().code()
    }
}
