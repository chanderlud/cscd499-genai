use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::RestartManager::RmEndSession;

fn call_rm_end_session() -> WIN32_ERROR {
    // RmEndSession returns WIN32_ERROR directly.
    // Using 0 as a concrete dummy session handle.
    unsafe { RmEndSession(0) }
}
