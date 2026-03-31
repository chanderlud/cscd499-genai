use windows::core::Error;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Security::WinTrust::OpenPersonalTrustDBDialog;

fn call_open_personal_trust_db_dialog() -> WIN32_ERROR {
    // SAFETY: OpenPersonalTrustDBDialog is a standard Win32 API. Passing None for the parent HWND is safe.
    let result = unsafe { OpenPersonalTrustDBDialog(None) };
    if result.0 != 0 {
        WIN32_ERROR(0)
    } else {
        let err = Error::from_thread();
        WIN32_ERROR::from_error(&err).unwrap_or(WIN32_ERROR(0))
    }
}
