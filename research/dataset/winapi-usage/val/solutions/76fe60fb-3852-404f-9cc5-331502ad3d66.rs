use windows::core::{Error, Result};
use windows::Win32::Foundation::{GetLastError, WIN32_ERROR};
use windows::Win32::Networking::WinHttp::{WinHttpAddRequestHeadersEx, WINHTTP_EXTENDED_HEADER};

fn call_win_http_add_request_headers_ex() -> WIN32_ERROR {
    let headers: [WINHTTP_EXTENDED_HEADER; 0] = [];
    let result = unsafe { WinHttpAddRequestHeadersEx(std::ptr::null_mut(), 0, 0, 0, &headers) };

    if result == 0 {
        unsafe { GetLastError() }
    } else {
        WIN32_ERROR(0)
    }
}
