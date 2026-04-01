use windows::core::{Error, BOOL};
use windows::Win32::UI::Magnification::MagGetFullscreenTransform;

fn call_mag_get_fullscreen_transform() -> windows::core::Result<BOOL> {
    let mut pmaglevel: f32 = 0.0;
    let mut pxoffset: i32 = 0;
    let mut pyoffset: i32 = 0;

    let result = unsafe { MagGetFullscreenTransform(&mut pmaglevel, &mut pxoffset, &mut pyoffset) };

    if result.0 != 0 {
        Ok(result)
    } else {
        Err(Error::from_thread())
    }
}
