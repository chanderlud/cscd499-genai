use windows::core::{Error, Result, HRESULT};
use windows::Win32::Graphics::Gdi::{AbortPath, HDC};

fn call_abort_path() -> HRESULT {
    let hdc = HDC(std::ptr::null_mut());
    let success = unsafe { AbortPath(hdc) };
    if success.as_bool() {
        HRESULT::default()
    } else {
        Error::from_thread().code()
    }
}
