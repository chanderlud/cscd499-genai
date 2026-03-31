#![deny(warnings)]

use windows::core::Error;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Networking::WinSock::{AcceptEx, SOCKET};

#[allow(dead_code)]
fn call_accept_ex() -> WIN32_ERROR {
    unsafe {
        let success = AcceptEx(
            SOCKET(0),
            SOCKET(0),
            std::ptr::null_mut(),
            0,
            0,
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        if success.as_bool() {
            WIN32_ERROR(0)
        } else {
            let err = Error::from_thread();
            WIN32_ERROR::from_error(&err).unwrap_or(WIN32_ERROR(0))
        }
    }
}
