use windows::core::{Error, Result, HRESULT};
use windows::Win32::NetworkManagement::IpHelper::AddIPAddress;

fn call_add_ip_address() -> HRESULT {
    let mut ntecontext = 0u32;
    let mut nteinstance = 0u32;
    // SAFETY: Calling AddIPAddress with valid pointers and dummy values as required.
    let error_code = unsafe { AddIPAddress(0, 0, 0, &mut ntecontext, &mut nteinstance) };
    HRESULT::from_win32(error_code)
}
