use windows::core::{Error, Result};
use windows::Win32::Foundation::HWND;
use windows::Win32::Security::WinTrust::WinVerifyTrust;

fn call_win_verify_trust() -> Result<i32> {
    let hwnd = HWND::default();
    let pgactionid = std::ptr::null_mut();
    let pwvtdata = std::ptr::null_mut();

    // SAFETY: WinVerifyTrust is an unsafe Win32 API. Passing null pointers is safe as the API
    // validates its arguments and returns an error HRESULT without causing undefined behavior.
    let result = unsafe { WinVerifyTrust(hwnd, pgactionid, pwvtdata) };
    Ok(result)
}
