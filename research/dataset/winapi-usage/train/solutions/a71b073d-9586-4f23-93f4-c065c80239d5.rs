use windows::core::{Error, Result};
use windows::Win32::Networking::WinHttp::WinHttpCheckPlatform;

fn call_win_http_check_platform() -> windows::core::HRESULT {
    unsafe {
        match WinHttpCheckPlatform() {
            Ok(()) => windows::core::HRESULT(0),
            Err(e) => e.code(),
        }
    }
}
