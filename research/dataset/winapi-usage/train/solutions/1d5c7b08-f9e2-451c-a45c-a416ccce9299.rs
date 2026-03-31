use windows::core::{Error, Result};
use windows::Win32::System::Environment::CreateEnclave;
use windows::Win32::System::Threading::GetCurrentProcess;

fn call_create_enclave() -> Result<*mut core::ffi::c_void> {
    // SAFETY: Calling CreateEnclave with valid parameters.
    let result = unsafe {
        let hprocess = GetCurrentProcess();
        CreateEnclave(hprocess, None, 0x1000, 0x1000, 0, std::ptr::null(), 0, None)
    };

    if result.is_null() {
        Err(Error::from_thread())
    } else {
        Ok(result)
    }
}
