use windows::Win32::Foundation::{GetLastError, WIN32_ERROR};
use windows::Win32::Security::WinTrust::OpenPersonalTrustDBDialogEx;

fn call_open_personal_trust_db_dialog_ex() -> windows::Win32::Foundation::WIN32_ERROR {
    // SAFETY: Win32 API call requires unsafe.
    let result = unsafe { OpenPersonalTrustDBDialogEx(None, 0, None) };

    if result.as_bool() {
        WIN32_ERROR(0)
    } else {
        // SAFETY: Win32 API call requires unsafe.
        unsafe { GetLastError() }
    }
}
