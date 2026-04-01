use windows::core::HRESULT;
use windows::Win32::System::Shutdown::{ExitWindowsEx, EXIT_WINDOWS_FLAGS, SHUTDOWN_REASON};

fn call_exit_windows_ex() -> HRESULT {
    let flags = EXIT_WINDOWS_FLAGS(0x00000000);
    let reason = SHUTDOWN_REASON(0x00000000);

    unsafe {
        ExitWindowsEx(flags, reason)
            .map(|_| windows::Win32::Foundation::S_OK)
            .unwrap_or_else(|e| e.code())
    }
}
