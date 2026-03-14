use windows::core::Result;
use windows::Win32::System::Threading::{
    GetCurrentProcess, SetPriorityClass, BELOW_NORMAL_PRIORITY_CLASS,
};

fn main() -> Result<()> {
    // Get the current process handle
    let process_handle = unsafe { GetCurrentProcess() };

    // Set the process priority to below normal
    // SetPriorityClass returns Result<()>, so we use the ? operator
    unsafe { SetPriorityClass(process_handle, BELOW_NORMAL_PRIORITY_CLASS)? };

    Ok(())
}
