#![deny(warnings)]

use windows::core::{Error, HRESULT};
use windows::Win32::Security::WinTrust::OpenPersonalTrustDBDialog;

#[allow(dead_code)]
fn call_open_personal_trust_db_dialog() -> HRESULT {
    let result = unsafe { OpenPersonalTrustDBDialog(None) };
    if result.as_bool() {
        HRESULT(0)
    } else {
        Error::from_thread().code()
    }
}
