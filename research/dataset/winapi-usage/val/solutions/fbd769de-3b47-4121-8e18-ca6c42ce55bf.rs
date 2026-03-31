use windows::core::{Error, Result};
use windows::Win32::Networking::WinHttp::{WinHttpAddRequestHeadersEx, WINHTTP_EXTENDED_HEADER};

fn call_win_http_add_request_headers_ex() -> Result<u32> {
    let headers: &[WINHTTP_EXTENDED_HEADER] = &[];
    // SAFETY: Calling WinHttpAddRequestHeadersEx with valid parameters.
    // A null handle and empty header slice are used for demonstration.
    let result = unsafe { WinHttpAddRequestHeadersEx(std::ptr::null_mut(), 0, 0, 0, headers) };
    if result == 0 {
        Err(Error::from_thread())
    } else {
        Ok(result)
    }
}
