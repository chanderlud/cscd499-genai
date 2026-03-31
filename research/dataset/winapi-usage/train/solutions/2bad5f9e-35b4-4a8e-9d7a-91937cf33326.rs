use windows::core::PCSTR;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Pipes::CallNamedPipeA;

fn call_call_named_pipe_a() -> WIN32_ERROR {
    let mut bytes_read: u32 = 0;
    // SAFETY: We provide a valid null-terminated ANSI string and a valid mutable pointer for the output parameter.
    let result = unsafe {
        CallNamedPipeA(
            PCSTR(b"\\\\.\\pipe\\test\0".as_ptr()),
            None,
            0,
            None,
            0,
            &mut bytes_read,
            0,
        )
    };
    match result {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
