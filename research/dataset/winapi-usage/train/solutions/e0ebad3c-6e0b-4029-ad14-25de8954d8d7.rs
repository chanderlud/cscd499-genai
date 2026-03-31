use windows::core::HRESULT;
use windows::Win32::Foundation::HWND;
use windows::Win32::Security::WinTrust::WinVerifyTrust;

fn call_win_verify_trust() -> HRESULT {
    unsafe {
        let code = WinVerifyTrust(
            HWND(std::ptr::null_mut()),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
        HRESULT(code)
    }
}
