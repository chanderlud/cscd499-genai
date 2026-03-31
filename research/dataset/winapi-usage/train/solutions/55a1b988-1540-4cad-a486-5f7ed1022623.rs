use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Networking::WinHttp::WinHttpAddRequestHeaders;

fn call_win_http_add_request_headers() -> WIN32_ERROR {
    let result = unsafe { WinHttpAddRequestHeaders(std::ptr::null_mut(), &[], 0) };
    match result {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
