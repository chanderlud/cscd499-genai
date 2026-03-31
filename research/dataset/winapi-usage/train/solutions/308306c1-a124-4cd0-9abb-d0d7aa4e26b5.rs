use windows::core::{Error, Result};
use windows::Win32::System::ProcessStatus::EnumDeviceDrivers;

fn call_enum_device_drivers() -> Result<()> {
    let mut buffer: [*mut core::ffi::c_void; 1024] = [core::ptr::null_mut(); 1024];
    let mut needed: u32 = 0;
    // SAFETY: buffer and needed are valid pointers with sufficient size for the API call.
    unsafe {
        EnumDeviceDrivers(
            buffer.as_mut_ptr(),
            (buffer.len() * std::mem::size_of::<*mut core::ffi::c_void>()) as u32,
            &mut needed,
        )?;
    }
    Ok(())
}
