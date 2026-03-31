use windows::core::{Error, Result};
use windows::Win32::Networking::WinSock::EnumProtocolsA;

fn call_enum_protocols_a() -> Result<i32> {
    let mut buffer_len: u32 = 0;
    // SAFETY: Passing null buffer and zero length to query required size is safe per API documentation.
    let result = unsafe { EnumProtocolsA(None, std::ptr::null_mut(), &mut buffer_len) };

    if result == -1 {
        Err(Error::from_thread())
    } else {
        Ok(result)
    }
}
