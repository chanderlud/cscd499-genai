use windows::core::{Error, Result};
use windows::Win32::Foundation::{GetLastError, WIN32_ERROR};
use windows::Win32::Networking::WinSock::EnumProtocolsA;

fn call_enum_protocols_a() -> WIN32_ERROR {
    let mut len = 0u32;
    let result = unsafe { EnumProtocolsA(None, std::ptr::null_mut(), &mut len) };

    if result == -1 {
        unsafe { GetLastError() }
    } else {
        WIN32_ERROR(0)
    }
}
