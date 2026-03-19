use windows::core::Result;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};

fn main() -> Result<()> {
    let target_pid = 1234; // Replace with actual PID
    let process_handle = unsafe { OpenProcess(PROCESS_TERMINATE, false, target_pid) }?;
    unsafe { TerminateProcess(process_handle, 0) }?;
    unsafe { CloseHandle(process_handle) }?;
    Ok(())
}
