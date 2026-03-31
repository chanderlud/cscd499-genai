use windows::core::BOOL;
use windows::core::{Error, Result};
use windows::Win32::Graphics::Gdi::{Arc, HDC};

fn call_arc() -> Result<BOOL> {
    let hdc = HDC(std::ptr::null_mut());
    // SAFETY: Arc is a standard Win32 GDI function. Passing a null HDC is safe;
    // it will fail gracefully and return FALSE, which we handle below.
    let result = unsafe { Arc(hdc, 0, 0, 100, 100, 50, 0, 0, 50) };
    if result == false {
        Err(Error::from_thread())
    } else {
        Ok(result)
    }
}
