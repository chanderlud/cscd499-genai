use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::RestartManager::RmCancelCurrentTask;

fn call_rm_cancel_current_task() -> WIN32_ERROR {
    // SAFETY: Calling the Win32 API with a concrete session handle value (0) as specified.
    unsafe { RmCancelCurrentTask(0) }
}
