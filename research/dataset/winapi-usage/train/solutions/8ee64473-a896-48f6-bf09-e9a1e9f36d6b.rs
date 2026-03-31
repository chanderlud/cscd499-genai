#[allow(unused_imports)]
use windows::core::{Error, Result, HRESULT};
use windows::Win32::System::Pipes::CallNamedPipeA;

fn call_call_named_pipe_a() -> HRESULT {
    let mut bytes_read: u32 = 0;
    let res: Result<()> = unsafe {
        CallNamedPipeA(
            windows::core::s!("\\\\.\\pipe\\test"),
            None,
            0,
            None,
            0,
            &mut bytes_read,
            0,
        )
    };
    res.map(|_| HRESULT(0)).unwrap_or_else(|e| e.code())
}
