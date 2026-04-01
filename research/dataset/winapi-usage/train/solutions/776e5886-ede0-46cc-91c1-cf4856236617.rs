use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::UI::Magnification::MagGetFullscreenTransform;

fn call_mag_get_fullscreen_transform() -> windows::Win32::Foundation::WIN32_ERROR {
    let mut pmaglevel: f32 = 0.0;
    let mut pxoffset: i32 = 0;
    let mut pyoffset: i32 = 0;

    let result = unsafe { MagGetFullscreenTransform(&mut pmaglevel, &mut pxoffset, &mut pyoffset) };

    if result.as_bool() {
        WIN32_ERROR(0)
    } else {
        let error = Error::from_thread();
        WIN32_ERROR(error.code().0 as u32)
    }
}
