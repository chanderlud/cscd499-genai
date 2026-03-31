use windows::core::{w, Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Networking::WinHttp::WinHttpConnect;

fn call_win_http_connect() -> WIN32_ERROR {
    unsafe {
        let hconnect = WinHttpConnect(std::ptr::null_mut(), w!("example.com"), 80, 0);
        if hconnect.is_null() {
            let err = Error::from_thread();
            WIN32_ERROR(err.code().0 as u32)
        } else {
            WIN32_ERROR(0)
        }
    }
}
