use windows::core::Result;
use windows::Win32::NetworkManagement::NetManagement::GetNetScheduleAccountInformation;

fn call_get_net_schedule_account_information() -> Result<()> {
    let mut account = [0u16; 256];
    // SAFETY: GetNetScheduleAccountInformation requires a valid mutable buffer for the account name.
    // We provide a sufficiently sized buffer and pass PCWSTR::null() for the server name to target the local machine.
    unsafe {
        GetNetScheduleAccountInformation(windows::core::PCWSTR::null(), &mut account)?;
    }
    Ok(())
}
