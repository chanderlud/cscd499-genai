use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Security::Authorization::{
    AuthzAccessCheck, AUTHZ_ACCESS_CHECK_FLAGS, AUTHZ_CLIENT_CONTEXT_HANDLE,
};
use windows::Win32::Security::PSECURITY_DESCRIPTOR;

fn call_authz_access_check() -> WIN32_ERROR {
    // SAFETY: Passing null/default values is safe for the API invocation itself.
    // The call will likely fail with an error code, which we capture and return.
    let result = unsafe {
        AuthzAccessCheck(
            AUTHZ_ACCESS_CHECK_FLAGS(0),
            AUTHZ_CLIENT_CONTEXT_HANDLE(std::ptr::null_mut()),
            std::ptr::null(),
            None,
            PSECURITY_DESCRIPTOR(std::ptr::null_mut()),
            None,
            std::ptr::null_mut(),
            None,
        )
    };

    match result {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
