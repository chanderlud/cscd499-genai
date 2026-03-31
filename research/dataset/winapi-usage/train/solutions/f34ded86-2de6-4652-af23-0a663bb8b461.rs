#![deny(warnings)]
#![allow(dead_code)]

use windows::core::{Result, BOOL};
use windows::Win32::System::Pipes::CallNamedPipeW;

fn call_call_named_pipe_w() -> Result<BOOL> {
    let mut bytes_read: u32 = 0;
    let pipe_name = windows::core::w!("\\\\.\\pipe\\testpipe");

    // SAFETY: CallNamedPipeW is invoked with a valid null-terminated wide string,
    // null input/output buffers with zero sizes, and a valid mutable pointer for bytes read.
    let result = unsafe { CallNamedPipeW(pipe_name, None, 0, None, 0, &mut bytes_read, 1000) };

    result.ok()?;
    Ok(result)
}
