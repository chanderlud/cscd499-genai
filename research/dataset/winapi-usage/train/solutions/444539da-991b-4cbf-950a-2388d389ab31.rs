use windows::core::{Error, Result};
use windows::Win32::Foundation::S_OK;
use windows::Win32::Networking::WinHttp::WinHttpAddRequestHeaders;

fn call_win_http_add_request_headers() -> windows::core::HRESULT {
    unsafe {
        match WinHttpAddRequestHeaders(std::ptr::null_mut(), &[], 0) {
            Ok(()) => S_OK,
            Err(e) => e.code(),
        }
    }
}
