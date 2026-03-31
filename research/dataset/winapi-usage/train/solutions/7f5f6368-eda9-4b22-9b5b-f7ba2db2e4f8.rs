use windows::core::{Error, Result};
use windows::Win32::NetworkManagement::NetManagement::I_NetLogonControl2;

fn call_i__net_logon_control2() -> windows::Win32::Foundation::WIN32_ERROR {
    let result = unsafe {
        I_NetLogonControl2(
            windows::core::PCWSTR::null(),
            0,
            0,
            std::ptr::null(),
            std::ptr::null_mut(),
        )
    };
    windows::Win32::Foundation::WIN32_ERROR(result)
}
