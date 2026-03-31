use windows::core::{Error, Result};
use windows::Win32::Foundation::{HANDLE, WIN32_ERROR};
use windows::Win32::System::IO::CreateIoCompletionPort;

fn call_create_io_completion_port() -> WIN32_ERROR {
    // SAFETY: Calling CreateIoCompletionPort with valid default parameters.
    match unsafe { CreateIoCompletionPort(HANDLE::default(), None, 0, 0) } {
        Ok(_) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
