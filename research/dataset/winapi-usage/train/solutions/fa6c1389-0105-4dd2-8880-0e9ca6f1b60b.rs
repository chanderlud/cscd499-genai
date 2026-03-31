use windows::core::{Error, Result};
use windows::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::System::IO::CreateIoCompletionPort;

fn call_create_io_completion_port() -> Result<HANDLE> {
    // SAFETY: Creating a new I/O completion port with standard parameters is safe.
    unsafe { CreateIoCompletionPort(INVALID_HANDLE_VALUE, None, 0, 0) }
}
