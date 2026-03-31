use windows::core::{Error, Result};
use windows::Win32::Security::WinTrust::OpenPersonalTrustDBDialog;

fn call_open_personal_trust_db_dialog() -> Result<windows::core::BOOL> {
    // SAFETY: OpenPersonalTrustDBDialog is a standard Win32 API.
    // Passing None for the parent window is safe and corresponds to a NULL HWND.
    let result = unsafe { OpenPersonalTrustDBDialog(None) };
    if result == false {
        Err(Error::from_thread())
    } else {
        Ok(result)
    }
}
