use windows::core::PCWSTR;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Security::Authentication::Identity::{AcquireCredentialsHandleW, SECPKG_CRED};
use windows::Win32::Security::Credentials::SecHandle;

fn call_acquire_credentials_handle_w() -> WIN32_ERROR {
    let mut cred_handle = SecHandle::default();
    let result = unsafe {
        AcquireCredentialsHandleW(
            PCWSTR::null(),
            PCWSTR::null(),
            SECPKG_CRED(0),
            None,
            None,
            None,
            None,
            &mut cred_handle,
            None,
        )
    };

    match result {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR::from_error(&e).unwrap_or_default(),
    }
}
