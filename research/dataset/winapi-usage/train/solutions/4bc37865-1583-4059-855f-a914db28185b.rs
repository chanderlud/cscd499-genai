use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};

fn call_get_system_power_status() -> windows::Win32::Foundation::WIN32_ERROR {
    let mut power_status = SYSTEM_POWER_STATUS::default();

    match unsafe { GetSystemPowerStatus(&mut power_status) } {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR::from_error(&e).unwrap_or(WIN32_ERROR(0)),
    }
}
