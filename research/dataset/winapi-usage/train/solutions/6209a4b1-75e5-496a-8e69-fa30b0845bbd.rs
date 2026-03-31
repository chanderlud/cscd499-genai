#[allow(unused_imports)]
use windows::core::{Error, Result, HRESULT};
use windows::Win32::Devices::Display::BRUSHOBJ_ulGetBrushColor;

fn call_brushobj_ul_get_brush_color() -> HRESULT {
    // SAFETY: Passing a null pointer as a concrete parameter value for this simple wrapper.
    let color = unsafe { BRUSHOBJ_ulGetBrushColor(std::ptr::null_mut()) };
    HRESULT(color as i32)
}
