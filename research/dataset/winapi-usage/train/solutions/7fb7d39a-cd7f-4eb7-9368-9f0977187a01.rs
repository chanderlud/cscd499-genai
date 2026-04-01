use windows::Win32::System::Shutdown::{
    ExitWindowsEx, EWX_LOGOFF, EXIT_WINDOWS_FLAGS, SHUTDOWN_REASON,
};

fn call_exit_windows_ex() -> windows::core::Result<()> {
    let flags = EWX_LOGOFF;
    let reason = SHUTDOWN_REASON(0);

    unsafe { ExitWindowsEx(flags, reason) }
}
