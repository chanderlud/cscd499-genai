use windows::core::{Error, Result};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Magnification::{MagGetColorEffect, MAGCOLOREFFECT};

fn call_mag_get_color_effect() -> Result<windows::core::BOOL> {
    let mut effect = MAGCOLOREFFECT::default();
    let hwnd = HWND::default();
    unsafe {
        let result = MagGetColorEffect(hwnd, &mut effect);
        if result.0 != 0 {
            Ok(result)
        } else {
            Err(Error::from_thread())
        }
    }
}
