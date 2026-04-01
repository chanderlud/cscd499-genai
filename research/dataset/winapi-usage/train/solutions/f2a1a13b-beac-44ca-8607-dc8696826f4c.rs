use windows::core::{Error, HRESULT};
use windows::Win32::Foundation::TRUE;
use windows::Win32::UI::Magnification::MagGetFullscreenTransform;

fn call_mag_get_fullscreen_transform() -> windows::core::HRESULT {
    let mut pmaglevel: f32 = 0.0;
    let mut pxoffset: i32 = 0;
    let mut pyoffset: i32 = 0;

    let result = unsafe { MagGetFullscreenTransform(&mut pmaglevel, &mut pxoffset, &mut pyoffset) };

    if result == TRUE {
        HRESULT(0)
    } else {
        Error::from_thread().code()
    }
}
