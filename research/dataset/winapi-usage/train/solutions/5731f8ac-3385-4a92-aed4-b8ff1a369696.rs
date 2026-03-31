use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Power::{CallNtPowerInformation, POWER_INFORMATION_LEVEL};

fn call_call_nt_power_information() -> WIN32_ERROR {
    let status = unsafe { CallNtPowerInformation(POWER_INFORMATION_LEVEL(0), None, 0, None, 0) };
    WIN32_ERROR(status.0 as u32)
}
