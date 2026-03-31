use windows::core::{Error, Result};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::IO::BindIoCompletionCallback;

fn call_bind_io_completion_callback() -> Result<()> {
    unsafe { BindIoCompletionCallback(HANDLE::default(), None, 0) }
}
