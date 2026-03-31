use windows::Win32::Foundation::{HANDLE, WIN32_ERROR};
use windows::Win32::System::Pipes::ConnectNamedPipe;

#[allow(dead_code)]
fn call_connect_named_pipe() -> WIN32_ERROR {
    match unsafe { ConnectNamedPipe(HANDLE::default(), None) } {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR::from_error(&e).unwrap_or(WIN32_ERROR(0)),
    }
}
