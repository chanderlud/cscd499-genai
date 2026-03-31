use windows::core::{Error, Result};
use windows::Win32::Devices::Display::BRUSHOBJ_hGetColorTransform;

fn call_brushobj_h_get_color_transform() -> windows::core::HRESULT {
    unsafe {
        BRUSHOBJ_hGetColorTransform(std::ptr::null_mut());
    }
    windows::core::HRESULT(0)
}
