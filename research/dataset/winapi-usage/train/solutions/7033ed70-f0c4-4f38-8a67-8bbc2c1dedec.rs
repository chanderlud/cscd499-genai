#[allow(unused_imports)]
use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Pipes::ConnectNamedPipe;

fn call_connect_named_pipe() -> HRESULT {
    // SAFETY: Calling with a null handle and no OVERLAPPED structure.
    // This will return an error, but safely demonstrates the API call.
    unsafe {
        match ConnectNamedPipe(HANDLE::default(), None) {
            Ok(()) => HRESULT::default(),
            Err(e) => e.code(),
        }
    }
}
