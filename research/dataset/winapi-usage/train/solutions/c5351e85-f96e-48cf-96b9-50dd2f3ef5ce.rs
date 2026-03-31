use windows::core::HRESULT;
use windows::core::PCSTR;
use windows::Win32::Security::Credentials::{CredDeleteA, CRED_TYPE};

fn call_cred_delete_a() -> HRESULT {
    match unsafe { CredDeleteA(PCSTR(b"test\0".as_ptr()), CRED_TYPE(0), None) } {
        Ok(()) => HRESULT(0),
        Err(e) => e.code(),
    }
}
