use windows::core::{Error, Result};
use windows::Win32::System::Power::DeletePwrScheme;

fn call_delete_pwr_scheme() -> Result<bool> {
    // SAFETY: DeletePwrScheme is a standard Win32 API. We pass 1 as a concrete power scheme ID.
    let success = unsafe { DeletePwrScheme(1) };
    if success {
        Ok(true)
    } else {
        Err(Error::from_thread())
    }
}
