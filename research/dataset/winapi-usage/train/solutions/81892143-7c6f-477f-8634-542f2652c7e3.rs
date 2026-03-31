#![allow(unused_imports)]
use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::NetworkManagement::IpHelper::AddIPAddress;

fn call_add_ip_address() -> WIN32_ERROR {
    let mut ntecontext: u32 = 0;
    let mut nteinstance: u32 = 0;
    let address: u32 = 0x0A000001;
    let ipmask: u32 = 0xFFFFFF00;
    let ifindex: u32 = 1;

    // SAFETY: AddIPAddress writes to the provided pointers. We pass valid mutable references.
    let error_code =
        unsafe { AddIPAddress(address, ipmask, ifindex, &mut ntecontext, &mut nteinstance) };

    WIN32_ERROR(error_code)
}
