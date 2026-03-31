#![allow(dead_code)]

use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Networking::WinHttp::WinHttpCloseHandle;

fn call_win_http_close_handle() -> WIN32_ERROR {
    unsafe {
        match WinHttpCloseHandle(std::ptr::null_mut()) {
            Ok(()) => WIN32_ERROR(0),
            Err(e) => WIN32_ERROR(e.code().0 as u32),
        }
    }
}
