use windows::core::{Error, Result};
use windows::Win32::Devices::Display::BRUSHOBJ_ulGetBrushColor;
use windows::Win32::Foundation::WIN32_ERROR;

fn call_brushobj_ul_get_brush_color() -> WIN32_ERROR {
    // SAFETY: The API requires a raw pointer. We pass null as a concrete value for this exercise.
    let color = unsafe { BRUSHOBJ_ulGetBrushColor(std::ptr::null_mut()) };
    WIN32_ERROR(color)
}
