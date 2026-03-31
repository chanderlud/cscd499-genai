use windows::core::{Error, Result, HRESULT};
use windows::Win32::Networking::WinHttp::WinHttpCloseHandle;

fn call_win_http_close_handle() -> HRESULT {
    unsafe {
        match WinHttpCloseHandle(std::ptr::null_mut::<std::ffi::c_void>()) {
            Ok(()) => HRESULT(0),
            Err(e) => e.code(),
        }
    }
}
