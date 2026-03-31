use windows::core::{Error, Result, HRESULT};
use windows::Win32::Networking::WinHttp::WinHttpConnect;

fn call_win_http_connect() -> HRESULT {
    let hconnect = unsafe {
        WinHttpConnect(
            std::ptr::null_mut(),
            windows::core::w!("example.com"),
            80,
            0,
        )
    };

    if hconnect.is_null() {
        Error::from_thread().code()
    } else {
        HRESULT(0)
    }
}
