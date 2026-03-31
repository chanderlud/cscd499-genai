use windows::core::w;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Security::Credentials::{CredDeleteW, CRED_TYPE};

fn call_cred_delete_w() -> WIN32_ERROR {
    // SAFETY: CredDeleteW is a Win32 API that requires unsafe due to raw pointer/string handling.
    let result = unsafe { CredDeleteW(w!("TestTarget"), CRED_TYPE(1), None) };
    match result {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR::from_error(&e).unwrap_or(WIN32_ERROR(0)),
    }
}
