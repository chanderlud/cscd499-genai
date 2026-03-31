use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Graphics::Gdi::{AbortPath, HDC};

fn call_abort_path() -> WIN32_ERROR {
    let hdc = HDC::default();
    let result = unsafe { AbortPath(hdc) };
    if result.as_bool() {
        WIN32_ERROR(0)
    } else {
        let err = Error::from_thread();
        WIN32_ERROR(err.code().0 as u32)
    }
}
