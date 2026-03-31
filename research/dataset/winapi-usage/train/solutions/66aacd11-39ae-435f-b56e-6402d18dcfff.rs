use windows::core::{Error, Result};
use windows::Win32::UI::Magnification::{MagGetFullscreenColorEffect, MAGCOLOREFFECT};

fn call_mag_get_fullscreen_color_effect() -> Result<windows::core::BOOL> {
    let mut effect = MAGCOLOREFFECT {
        transform: [0.0; 25],
    };
    // SAFETY: `effect` is a valid mutable pointer to a MAGCOLOREFFECT struct.
    let result = unsafe { MagGetFullscreenColorEffect(&mut effect) };
    result.ok()?;
    Ok(result)
}
