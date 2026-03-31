use windows::core::{Error, Result, HRESULT};
use windows::Win32::System::RestartManager::RmCancelCurrentTask;

fn call_rm_cancel_current_task() -> HRESULT {
    let err = unsafe { RmCancelCurrentTask(0) };
    HRESULT::from_win32(err.0)
}
