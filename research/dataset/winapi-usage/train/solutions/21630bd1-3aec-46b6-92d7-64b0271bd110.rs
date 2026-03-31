use windows::core::{Error, Result};
use windows::Win32::Security::{FreeSid, PSID};

fn call_free_sid() -> Result<*mut core::ffi::c_void> {
    let psid = PSID(std::ptr::null_mut());
    // FreeSid returns PVOID (always NULL on success/failure), so we wrap it directly.
    let result = unsafe { FreeSid(psid) };
    Ok(result)
}
