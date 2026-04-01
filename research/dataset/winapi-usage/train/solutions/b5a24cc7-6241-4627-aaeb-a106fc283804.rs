use windows::Win32::Foundation::{ERROR_SUCCESS, WIN32_ERROR};
use windows::Win32::System::Shutdown::{
    ExitWindowsEx, EWX_SHUTDOWN, EXIT_WINDOWS_FLAGS, SHTDN_REASON_FLAG_PLANNED, SHUTDOWN_REASON,
};

fn call_exit_windows_ex() -> windows::Win32::Foundation::WIN32_ERROR {
    // Call ExitWindowsEx with concrete parameter values
    let result = unsafe {
        ExitWindowsEx(
            EXIT_WINDOWS_FLAGS(EWX_SHUTDOWN.0),
            SHUTDOWN_REASON(SHTDN_REASON_FLAG_PLANNED.0),
        )
    };

    // Convert Result to WIN32_ERROR
    match result {
        Ok(()) => WIN32_ERROR(ERROR_SUCCESS.0),
        Err(e) => WIN32_ERROR::from_error(&e).unwrap_or(ERROR_SUCCESS),
    }
}
