use windows::core::{w, Error, Result, HRESULT};
use windows::Win32::System::Pipes::CallNamedPipeW;

fn call_call_named_pipe_w() -> HRESULT {
    let mut bytes_read: u32 = 0;
    // SAFETY: We pass valid pointers and sizes. The pipe name is a valid wide string.
    let success = unsafe {
        CallNamedPipeW(
            w!("\\\\.\\pipe\\test"),
            None,
            0,
            None,
            0,
            &mut bytes_read,
            0,
        )
    };

    if success.as_bool() {
        HRESULT(0)
    } else {
        Error::from_thread().code()
    }
}
