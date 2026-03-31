use windows::core::{Error, Result};
use windows::Win32::System::SystemInformation::{GetSystemInfo, SYSTEM_INFO};

fn call_get_system_info() -> Result<()> {
    let mut info = SYSTEM_INFO::default();
    // SAFETY: GetSystemInfo expects a valid mutable pointer to a SYSTEM_INFO struct.
    unsafe { GetSystemInfo(&mut info) };
    Ok(())
}
