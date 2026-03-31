use windows::core::{Error, Result};
use windows::Win32::Devices::Display::BRUSHOBJ_hGetColorTransform;
use windows::Win32::Foundation::WIN32_ERROR;

fn call_brushobj_h_get_color_transform() -> windows::Win32::Foundation::WIN32_ERROR {
    unsafe {
        BRUSHOBJ_hGetColorTransform(std::ptr::null_mut());
    }
    WIN32_ERROR(0)
}
