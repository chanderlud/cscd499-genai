use windows::core::{Error, Result};
use windows::Win32::NetworkManagement::NetManagement::LogErrorA;

fn call_log_error_a() -> Result<()> {
    unsafe {
        LogErrorA(0, &[], 0);
    }
    Ok(())
}
