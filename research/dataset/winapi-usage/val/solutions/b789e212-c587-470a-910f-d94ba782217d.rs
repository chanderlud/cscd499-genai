use windows::core::{Error, Result, HRESULT};
use windows::Win32::Networking::WinSock::EnumProtocolsA;

fn call_enum_protocols_a() -> HRESULT {
    let mut buffer_length = 0u32;
    // SAFETY: Passing null buffer and length pointer to query required size is a standard and safe usage pattern.
    let result = unsafe { EnumProtocolsA(None, std::ptr::null_mut(), &mut buffer_length) };
    if result == -1 {
        Error::from_thread().code()
    } else {
        HRESULT::from_win32(0)
    }
}
