#![deny(warnings)]

use windows::core::Result;
use windows::Win32::Networking::WinHttp::WinHttpAddRequestHeaders;

#[allow(dead_code)]
fn call_win_http_add_request_headers() -> Result<()> {
    let headers = windows::core::w!("Content-Type: application/json\r\n");
    unsafe { WinHttpAddRequestHeaders(std::ptr::null_mut(), headers.as_wide(), 0) }
}
