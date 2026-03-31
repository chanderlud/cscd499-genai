use windows::core::{Error, Result};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Pipes::ConnectNamedPipe;

fn call_connect_named_pipe() -> Result<Result<()>> {
    let handle = HANDLE::default();
    Ok(unsafe { ConnectNamedPipe(handle, None) })
}
