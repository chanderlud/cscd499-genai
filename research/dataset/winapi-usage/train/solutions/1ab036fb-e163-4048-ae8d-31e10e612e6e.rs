use windows::core::{w, Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Pipes::CallNamedPipeW;

fn call_call_named_pipe_w() -> WIN32_ERROR {
    let mut bytes_read: u32 = 0;
    let result = unsafe {
        CallNamedPipeW(
            w!("\\\\.\\pipe\\testpipe"),
            None,
            0,
            None,
            0,
            &mut bytes_read,
            1000,
        )
    };

    if result.as_bool() {
        WIN32_ERROR(0)
    } else {
        let err = Error::from_thread();
        WIN32_ERROR(err.code().0 as u32)
    }
}
