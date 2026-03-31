use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::RestartManager::RmEndSession;

fn call_rm_end_session() -> Result<WIN32_ERROR> {
    // SAFETY: Calling RmEndSession with a dummy session handle (0).
    // The API returns a WIN32_ERROR directly, which we validate.
    let result = unsafe { RmEndSession(0) };
    if result.0 != 0 {
        return Err(Error::from_hresult(result.to_hresult()));
    }
    Ok(result)
}
