use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::{CloseHandle, HANDLE};

fn call_close_handle() -> HRESULT {
    match unsafe { CloseHandle(HANDLE::default()) } {
        Ok(()) => HRESULT::default(),
        Err(e) => e.code(),
    }
}
