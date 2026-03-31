use windows::core::PCSTR;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Security::Credentials::{CredDeleteA, CRED_TYPE};

fn call_cred_delete_a() -> WIN32_ERROR {
    let result = unsafe {
        CredDeleteA(
            PCSTR::from_raw(b"TestTarget\0".as_ptr()),
            CRED_TYPE(0),
            None,
        )
    };
    match result {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR::from_error(&e).unwrap_or_else(|| WIN32_ERROR(e.code().0 as u32)),
    }
}
