use windows::core::{Error, Result};
use windows::Win32::Foundation::{HWND, WIN32_ERROR};
use windows::Win32::Security::WinTrust::WinVerifyTrust;

fn call_win_verify_trust() -> WIN32_ERROR {
    unsafe {
        let result = WinVerifyTrust(HWND::default(), std::ptr::null_mut(), std::ptr::null_mut());
        WIN32_ERROR(result as u32)
    }
}
