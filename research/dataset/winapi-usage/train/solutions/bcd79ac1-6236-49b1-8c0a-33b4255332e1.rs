use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::{ERROR_SUCCESS, WIN32_ERROR};
use windows::Win32::System::RestartManager::RmCancelCurrentTask;

fn call_rm_cancel_current_task() -> Result<WIN32_ERROR> {
    // SAFETY: Calling RmCancelCurrentTask with a dummy session handle (0).
    let result = unsafe { RmCancelCurrentTask(0) };
    if result != ERROR_SUCCESS {
        return Err(Error::from_hresult(HRESULT::from_win32(result.0)));
    }
    Ok(result)
}
