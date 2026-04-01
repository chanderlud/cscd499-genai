use windows::core::{Error, Result, HRESULT};
use windows::Win32::Networking::WinSock::EnumProtocolsW;

fn call_enum_protocols_w() -> windows::core::HRESULT {
    let mut buffer_length: u32 = 0;
    let result = unsafe { EnumProtocolsW(None, std::ptr::null_mut(), &mut buffer_length) };

    if result == -1 {
        Error::from_thread().code()
    } else {
        HRESULT::from_win32(0)
    }
}
