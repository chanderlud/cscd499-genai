use windows::Win32::Foundation::{HANDLE, WIN32_ERROR};
use windows::Win32::System::IO::BindIoCompletionCallback;

fn call_bind_io_completion_callback() -> WIN32_ERROR {
    let result = unsafe { BindIoCompletionCallback(HANDLE::default(), None, 0) };
    match result {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR::from_error(&e).unwrap_or_default(),
    }
}
