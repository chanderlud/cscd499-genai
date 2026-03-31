use windows::core::{Error, Result, PCSTR};
use windows::Win32::System::Pipes::CallNamedPipeA;

fn call_call_named_pipe_a() -> Result<()> {
    let mut bytes_read: u32 = 0;
    unsafe {
        CallNamedPipeA(
            PCSTR(b"\\\\.\\pipe\\testpipe\0".as_ptr()),
            None,
            0,
            None,
            0,
            &mut bytes_read,
            1000,
        )?;
    }
    Ok(())
}
