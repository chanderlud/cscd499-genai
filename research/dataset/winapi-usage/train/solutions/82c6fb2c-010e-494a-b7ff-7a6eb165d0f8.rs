use windows::core::{Error, Result};
use windows::Win32::Foundation::NTSTATUS;
use windows::Win32::System::Power::{CallNtPowerInformation, POWER_INFORMATION_LEVEL};

fn call_call_nt_power_information() -> Result<NTSTATUS> {
    let status = unsafe { CallNtPowerInformation(POWER_INFORMATION_LEVEL(0), None, 0, None, 0) };
    status.ok()?;
    Ok(status)
}
