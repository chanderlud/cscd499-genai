use windows::core::{Error, Result};
use windows::Win32::Networking::WinSock::EnumProtocolsW;

fn call_enum_protocols_w() -> windows::core::Result<i32> {
    // Buffer to receive protocol information
    let mut buffer: Vec<u8> = vec![0u8; 1024];
    let mut buffer_length: u32 = buffer.len() as u32;

    // Call EnumProtocolsW with concrete parameters
    // None for lpiprotocols to get all protocols
    let result = unsafe {
        EnumProtocolsW(
            None,
            buffer.as_mut_ptr() as *mut core::ffi::c_void,
            &mut buffer_length,
        )
    };

    // EnumProtocolsW returns -1 on error, otherwise returns count of protocols
    if result == -1 {
        Err(Error::from_thread())
    } else {
        Ok(result)
    }
}
