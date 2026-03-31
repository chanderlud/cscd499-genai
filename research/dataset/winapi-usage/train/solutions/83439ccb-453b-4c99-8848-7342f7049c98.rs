use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Power::DeletePwrScheme;

fn call_delete_pwr_scheme() -> WIN32_ERROR {
    // SAFETY: DeletePwrScheme is a Win32 API that takes a power scheme ID.
    // We pass 1 as a concrete example. It returns false on failure.
    if unsafe { DeletePwrScheme(1) } {
        WIN32_ERROR(0)
    } else {
        let err = Error::from_thread();
        WIN32_ERROR(err.code().0 as u32)
    }
}
