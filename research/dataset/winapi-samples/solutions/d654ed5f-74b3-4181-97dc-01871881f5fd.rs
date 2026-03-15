use windows::core::Result;
use windows::Win32::System::Threading::{BELOW_NORMAL_PRIORITY_CLASS, GetCurrentProcess, SetPriorityClass};

fn main() -> Result<()> {
    // Get the current process handle safely without unsafe block in the function body
    let process_handle = unsafe { GetCurrentProcess() };

    // Use ? for error propagation and keep the unsafe call minimal
    unsafe { SetPriorityClass(process_handle, BELOW_NORMAL_PRIORITY_CLASS) }?;

    Ok(())
}
