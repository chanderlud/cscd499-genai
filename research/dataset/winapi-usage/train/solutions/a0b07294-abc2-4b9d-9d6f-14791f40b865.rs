use windows::Win32::Foundation::{HANDLE, WIN32_ERROR};
use windows::Win32::System::IO::CancelIo;

#[allow(dead_code)]
fn call_cancel_io() -> WIN32_ERROR {
    match unsafe { CancelIo(HANDLE::default()) } {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR::from_error(&e).unwrap_or_else(|| WIN32_ERROR(e.code().0 as u32)),
    }
}
