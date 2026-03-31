use windows::core::{Error, Result, HRESULT};
use windows::Win32::NetworkManagement::NetManagement::I_NetLogonControl2;

fn call_i__net_logon_control2() -> HRESULT {
    let status = unsafe {
        I_NetLogonControl2(
            windows::core::PCWSTR::null(),
            0,
            0,
            std::ptr::null(),
            std::ptr::null_mut(),
        )
    };
    HRESULT::from_win32(status)
}
