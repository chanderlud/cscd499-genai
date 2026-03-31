use windows::core::{HRESULT, PCWSTR};
use windows::Win32::NetworkManagement::NetManagement::GetNetScheduleAccountInformation;

fn call_get_net_schedule_account_information() -> HRESULT {
    let mut account = [0u16; 256];
    unsafe {
        match GetNetScheduleAccountInformation(PCWSTR::null(), &mut account) {
            Ok(()) => HRESULT(0),
            Err(e) => e.code(),
        }
    }
}
