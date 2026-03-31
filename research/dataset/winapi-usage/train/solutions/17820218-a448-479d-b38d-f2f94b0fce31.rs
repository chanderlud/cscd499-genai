use windows::core::{Error, HRESULT};
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Magnification::{MagGetColorEffect, MAGCOLOREFFECT};

fn call_mag_get_color_effect() -> HRESULT {
    let mut effect = MAGCOLOREFFECT {
        transform: [0.0; 25],
    };
    let success = unsafe { MagGetColorEffect(HWND::default(), &mut effect) };
    if success.as_bool() {
        HRESULT(0)
    } else {
        Error::from_thread().code()
    }
}
