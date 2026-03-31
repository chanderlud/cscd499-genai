use windows::core::{Error, Result};
use windows::Win32::System::Power::CanUserWritePwrScheme;

fn call_can_user_write_pwr_scheme() -> Result<bool> {
    // SAFETY: CanUserWritePwrScheme takes no parameters and safely returns a boolean indicating permission.
    Ok(unsafe { CanUserWritePwrScheme() })
}
