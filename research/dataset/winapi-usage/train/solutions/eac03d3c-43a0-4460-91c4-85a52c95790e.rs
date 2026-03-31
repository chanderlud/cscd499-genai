use windows::core::{Error, Result};
use windows::Win32::Devices::Display::{BRUSHOBJ_pvGetRbrush, BRUSHOBJ};

fn call_brushobj_pv_get_rbrush() -> Result<*mut core::ffi::c_void> {
    let mut brush_obj: BRUSHOBJ = unsafe { std::mem::zeroed() };
    let ptr = unsafe { BRUSHOBJ_pvGetRbrush(&mut brush_obj) };
    Ok(ptr)
}
