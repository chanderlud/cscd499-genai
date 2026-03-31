use windows::core::{Error, Result, HRESULT};
use windows::Win32::Security::WinTrust::OpenPersonalTrustDBDialogEx;

fn call_open_personal_trust_db_dialog_ex() -> HRESULT {
    unsafe {
        let success = OpenPersonalTrustDBDialogEx(None, 0, None);
        if success.as_bool() {
            HRESULT::from_win32(0)
        } else {
            Error::from_thread().code()
        }
    }
}
