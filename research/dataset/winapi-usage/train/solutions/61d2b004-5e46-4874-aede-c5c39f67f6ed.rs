use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};

fn call_get_system_power_status() -> windows::core::Result<windows::core::Result<()>> {
    let mut power_status = SYSTEM_POWER_STATUS::default();
    let result = unsafe { GetSystemPowerStatus(&mut power_status) }?;

    Ok(Ok(()))
}
