use windows::core::PCWSTR;
#[allow(unused_imports)]
use windows::core::{Error, Result};
use windows::Win32::System::Shutdown::AbortSystemShutdownW;

fn call_abort_system_shutdown_w() -> Result<()> {
    // SAFETY: Passing a null PCWSTR is documented to target the local machine.
    // The Win32 API returns a Result, allowing direct use of the `?` operator.
    unsafe {
        AbortSystemShutdownW(PCWSTR::null())?;
    }
    Ok(())
}
