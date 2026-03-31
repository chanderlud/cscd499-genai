#[allow(unused_imports)]
use windows::core::{w, Error, Result, HRESULT};
use windows::Win32::Security::Credentials::{CredDeleteW, CRED_TYPE};

fn call_cred_delete_w() -> HRESULT {
    unsafe {
        match CredDeleteW(w!("test_target"), CRED_TYPE(0), Some(0)) {
            Ok(()) => HRESULT(0),
            Err(e) => e.code(),
        }
    }
}
