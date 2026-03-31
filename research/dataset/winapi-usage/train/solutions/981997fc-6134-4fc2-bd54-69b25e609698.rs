use windows::core::{Error, Result, HRESULT};
use windows::Win32::Security::Authentication::Identity::{AcquireCredentialsHandleW, SECPKG_CRED};
use windows::Win32::Security::Credentials::SecHandle;

fn call_acquire_credentials_handle_w() -> windows::core::HRESULT {
    let mut cred_handle = SecHandle::default();
    let result = unsafe {
        AcquireCredentialsHandleW(
            windows::core::PCWSTR::null(),
            windows::core::PCWSTR::null(),
            SECPKG_CRED(0),
            None,
            None,
            None,
            None,
            &mut cred_handle,
            None,
        )
    };
    result
        .map(|_| HRESULT::default())
        .unwrap_or_else(|e| e.code())
}
