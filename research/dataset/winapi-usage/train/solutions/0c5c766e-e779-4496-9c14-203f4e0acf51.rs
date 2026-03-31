use windows::core::{Error, Result};
use windows::Win32::NetworkManagement::NetManagement::LogErrorW;

fn call_log_error_w() -> Result<()> {
    // SAFETY: LogErrorW is a Win32 API that logs an error to the event log.
    // Passing an empty slice for substrings and 0 for message ID and error code is safe.
    unsafe {
        LogErrorW(0, &[], 0);
    }
    Ok(())
}
