use windows::core::{HRESULT, PCSTR};
use windows::Win32::Security::Credentials::CredEnumerateA;

fn call_cred_enumerate_a() -> windows::core::HRESULT {
    unsafe {
        CredEnumerateA(
            PCSTR::null(),
            None,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
        .map(|_| HRESULT(0))
        .unwrap_or_else(|e| e.code())
    }
}
