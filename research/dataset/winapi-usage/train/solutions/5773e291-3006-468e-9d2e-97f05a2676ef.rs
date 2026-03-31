#![allow(dead_code)]
use windows::core::{Error, Result};
use windows::Win32::Security::WinTrust::OpenPersonalTrustDBDialogEx;

fn call_open_personal_trust_db_dialog_ex() -> Result<windows::core::BOOL> {
    // SAFETY: Passing None for HWND and reserved pointer, and 0 for flags is valid and safe.
    let result = unsafe { OpenPersonalTrustDBDialogEx(None, 0, None) };
    if result.0 == 0 {
        return Err(Error::from_thread());
    }
    Ok(result)
}
