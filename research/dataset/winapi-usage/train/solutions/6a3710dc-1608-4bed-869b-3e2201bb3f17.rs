use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::IO::CancelIo;

fn call_cancel_io() -> HRESULT {
    unsafe {
        match CancelIo(HANDLE::default()) {
            Ok(()) => HRESULT::default(),
            Err(e) => e.code(),
        }
    }
}
