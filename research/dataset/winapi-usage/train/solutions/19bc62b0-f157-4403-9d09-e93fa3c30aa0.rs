use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Environment::CreateEnvironmentBlock;

fn call_create_environment_block() -> Result<()> {
    let mut environment: *mut core::ffi::c_void = std::ptr::null_mut();

    unsafe {
        CreateEnvironmentBlock(&mut environment as *mut *mut core::ffi::c_void, None, true)?;
    }

    Ok(())
}
