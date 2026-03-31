use windows::core::{Error, Result};
use windows::Win32::Graphics::Gdi::{AbortPath, HDC};

fn call_abort_path() -> Result<windows::core::BOOL> {
    let hdc = HDC(std::ptr::null_mut());
    let result = unsafe { AbortPath(hdc) };
    if result == false {
        Err(Error::from_thread())
    } else {
        Ok(result)
    }
}
