use windows::core::{Error, Result, HRESULT};
use windows::Win32::Networking::WinHttp::WinHttpAddRequestHeadersEx;

fn call_win_http_add_request_headers_ex() -> HRESULT {
    // SAFETY: Passing null/empty parameters is safe for this API call demonstration.
    let result = unsafe { WinHttpAddRequestHeadersEx(std::ptr::null_mut(), 0, 0, 0, &[]) };
    if result == 0 {
        Error::from_thread().code()
    } else {
        HRESULT(0)
    }
}
