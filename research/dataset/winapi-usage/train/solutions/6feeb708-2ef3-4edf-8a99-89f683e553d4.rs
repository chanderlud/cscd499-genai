#![deny(warnings)]

use windows::Win32::Foundation::{HANDLE, WIN32_ERROR};
use windows::Win32::NetworkManagement::IpHelper::CancelMibChangeNotify2;

#[allow(dead_code)]
fn call_cancel_mib_change_notify2() -> WIN32_ERROR {
    unsafe { CancelMibChangeNotify2(HANDLE::default()) }
}
