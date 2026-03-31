use windows::core::{HRESULT, PCWSTR};
use windows::Win32::Security::Credentials::{CredEnumerateW, CREDENTIALW, CRED_ENUMERATE_FLAGS};

fn call_cred_enumerate_w() -> HRESULT {
    let mut count = 0u32;
    let mut credential: *mut *mut CREDENTIALW = std::ptr::null_mut();
    // SAFETY: We provide valid mutable references for out-parameters as required by the API.
    unsafe {
        match CredEnumerateW(
            PCWSTR::null(),
            None::<CRED_ENUMERATE_FLAGS>,
            &mut count,
            &mut credential,
        ) {
            Ok(()) => HRESULT(0),
            Err(e) => e.code(),
        }
    }
}
