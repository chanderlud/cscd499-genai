use windows::core::PCWSTR;
use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::NetworkManagement::NetManagement::GetNetScheduleAccountInformation;

fn call_get_net_schedule_account_information() -> WIN32_ERROR {
    let mut account = [0u16; 256];
    // SAFETY: GetNetScheduleAccountInformation writes to the provided buffer.
    // We pass a valid mutable slice and PCWSTR::null() for the server name (local machine).
    match unsafe { GetNetScheduleAccountInformation(PCWSTR::null(), &mut account) } {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR::from_error(&e).unwrap_or(WIN32_ERROR(0)),
    }
}
