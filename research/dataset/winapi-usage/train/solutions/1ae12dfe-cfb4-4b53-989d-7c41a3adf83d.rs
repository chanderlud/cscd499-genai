use windows::core::{Error, Result};
use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};

fn call_get_system_power_status() -> windows::core::HRESULT {
    let mut status = unsafe { std::mem::zeroed() };
    match unsafe { GetSystemPowerStatus(&mut status) } {
        Ok(()) => windows::core::HRESULT(0),
        Err(e) => e.code(),
    }
}
