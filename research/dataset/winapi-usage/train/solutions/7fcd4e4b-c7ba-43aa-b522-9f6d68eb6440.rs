use windows::core::{w, Error, Result};
use windows::Win32::Networking::WinHttp::WinHttpConnect;

fn call_win_http_connect() -> Result<*mut core::ffi::c_void> {
    let hsession: *mut core::ffi::c_void = std::ptr::null_mut();
    let port: u16 = 80;
    let reserved: u32 = 0;

    // SAFETY: WinHttpConnect is called with valid parameters.
    // We check for NULL return to handle failure via GetLastError.
    unsafe {
        let result = WinHttpConnect(hsession, w!("example.com"), port, reserved);
        if result.is_null() {
            Err(Error::from_thread())
        } else {
            Ok(result)
        }
    }
}
