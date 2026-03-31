use windows::core::{Error, Result};
use windows::Win32::Devices::Display::{BRUSHOBJ_ulGetBrushColor, BRUSHOBJ};

fn call_brushobj_ul_get_brush_color() -> Result<u32> {
    let mut brush_obj = BRUSHOBJ::default();
    // SAFETY: Passing a valid mutable pointer to a BRUSHOBJ as required by the API contract.
    let color = unsafe { BRUSHOBJ_ulGetBrushColor(&mut brush_obj) };
    Ok(color)
}
