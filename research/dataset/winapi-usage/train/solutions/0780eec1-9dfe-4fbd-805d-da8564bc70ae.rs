use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::IO::CreateIoCompletionPort;

fn call_create_io_completion_port() -> HRESULT {
    // SAFETY: Calling with default/null handles is safe for demonstration purposes.
    unsafe {
        match CreateIoCompletionPort(HANDLE::default(), None, 0, 0) {
            Ok(_) => HRESULT::default(),
            Err(e) => e.code(),
        }
    }
}
