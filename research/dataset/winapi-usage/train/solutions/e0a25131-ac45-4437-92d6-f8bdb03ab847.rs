use windows::core::{Error, Result, HRESULT, PCSTR};
use windows::Win32::Foundation::S_OK;
use windows::Win32::Security::Authentication::Identity::{AcquireCredentialsHandleA, SECPKG_CRED};
use windows::Win32::Security::Credentials::SecHandle;

fn call_acquire_credentials_handle_a() -> HRESULT {
    let mut cred_handle = SecHandle {
        dwLower: 0,
        dwUpper: 0,
    };
    unsafe {
        AcquireCredentialsHandleA(
            PCSTR::null(),
            PCSTR::null(),
            SECPKG_CRED(2),
            None,
            None,
            None,
            None,
            &mut cred_handle,
            None,
        )
    }
    .map(|_| S_OK)
    .unwrap_or_else(|e: Error| e.code())
}
