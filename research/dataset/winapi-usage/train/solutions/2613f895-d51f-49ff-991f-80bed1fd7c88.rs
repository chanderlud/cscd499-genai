use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::IO::BindIoCompletionCallback;

fn call_bind_io_completion_callback() -> HRESULT {
    unsafe {
        match BindIoCompletionCallback(HANDLE::default(), None, 0) {
            Ok(()) => HRESULT::default(),
            Err(e) => e.code(),
        }
    }
}
