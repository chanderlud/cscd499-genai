use windows::core::{Error, Result, BOOL};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Graphics::Gdi::{Arc, HDC};

fn call_arc() -> WIN32_ERROR {
    let hdc = HDC::default();
    let result: BOOL = unsafe { Arc(hdc, 0, 0, 100, 100, 0, 0, 100, 100) };
    if result.as_bool() {
        WIN32_ERROR(0)
    } else {
        let err = Error::from_thread();
        WIN32_ERROR(err.code().0 as u32)
    }
}
