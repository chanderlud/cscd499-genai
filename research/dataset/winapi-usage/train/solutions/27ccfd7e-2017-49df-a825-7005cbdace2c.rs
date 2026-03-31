use windows::core::{Error, Result};
use windows::Win32::Networking::WinSock::{AcceptEx, SOCKET};

fn call_accept_ex() -> Result<windows::core::BOOL> {
    // SAFETY: AcceptEx is a Win32 API that requires valid socket handles and buffers.
    // We pass null/zero values as concrete placeholders per the task requirements.
    let result = unsafe {
        AcceptEx(
            SOCKET(0),
            SOCKET(0),
            std::ptr::null_mut(),
            0,
            0,
            0,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if !result.as_bool() {
        return Err(Error::from_thread());
    }
    Ok(result)
}
