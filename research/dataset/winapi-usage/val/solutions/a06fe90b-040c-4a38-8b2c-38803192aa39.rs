use windows::core::{Error, Result};
use windows::Win32::Devices::Display::{BRUSHOBJ_hGetColorTransform, BRUSHOBJ};
use windows::Win32::Foundation::HANDLE;

fn call_brushobj_h_get_color_transform() -> Result<HANDLE> {
    let pbo: *mut BRUSHOBJ = std::ptr::null_mut();
    // SAFETY: Calling the Win32 API with a null pointer as a concrete parameter value for this exercise.
    let handle = unsafe { BRUSHOBJ_hGetColorTransform(pbo) };
    if handle.is_invalid() {
        return Err(Error::from_thread());
    }
    Ok(handle)
}
