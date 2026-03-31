use windows::core::{Error, Result};
use windows::Win32::Devices::Display::{BRUSHOBJ_pvAllocRbrush, BRUSHOBJ};

fn call_brushobj_pv_alloc_rbrush() -> Result<*mut core::ffi::c_void> {
    let mut brush_obj = BRUSHOBJ::default();
    // SAFETY: We pass a valid mutable pointer to a BRUSHOBJ instance.
    let ptr =
        unsafe { BRUSHOBJ_pvAllocRbrush(&mut brush_obj, std::mem::size_of::<BRUSHOBJ>() as u32) };
    if ptr.is_null() {
        return Err(Error::from_thread());
    }
    Ok(ptr)
}
