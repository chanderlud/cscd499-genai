use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Security::Authentication::Identity::{AcquireCredentialsHandleA, SECPKG_CRED};
use windows::Win32::Security::Credentials::SecHandle;

fn call_acquire_credentials_handle_a() -> WIN32_ERROR {
    let mut cred_handle = SecHandle {
        dwLower: 0,
        dwUpper: 0,
    };
    unsafe {
        match AcquireCredentialsHandleA(
            None,
            None,
            SECPKG_CRED(0),
            None,
            None,
            None,
            None,
            &mut cred_handle,
            None,
        ) {
            Ok(()) => WIN32_ERROR(0),
            Err(e) => WIN32_ERROR(e.code().0 as u32),
        }
    }
}
