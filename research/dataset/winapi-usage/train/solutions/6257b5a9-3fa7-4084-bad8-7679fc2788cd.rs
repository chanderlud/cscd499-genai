use windows::core::{Error, Result, HRESULT};
use windows::Win32::NetworkManagement::IpHelper::AddIPAddress;

fn call_add_ip_address() -> Result<u32> {
    let mut ntecontext: u32 = 0;
    let mut nteinstance: u32 = 0;
    let address: u32 = 0xC0A80001;
    let ipmask: u32 = 0xFFFFFF00;
    let ifindex: u32 = 1;

    // SAFETY: AddIPAddress requires valid mutable pointers for output parameters.
    // We provide valid stack-allocated u32 variables.
    let result =
        unsafe { AddIPAddress(address, ipmask, ifindex, &mut ntecontext, &mut nteinstance) };

    if result == 0 {
        Ok(ntecontext)
    } else {
        Err(Error::from_hresult(HRESULT::from_win32(result)))
    }
}
