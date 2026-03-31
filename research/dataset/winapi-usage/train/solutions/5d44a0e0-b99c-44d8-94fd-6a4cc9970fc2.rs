use windows::core::{Error, Result};
use windows::Win32::System::Shutdown::AbortSystemShutdownA;

fn call_abort_system_shutdown_a() -> Result<()> {
    unsafe {
        AbortSystemShutdownA(windows::core::PCSTR::null())?;
    }
    Ok(())
}
