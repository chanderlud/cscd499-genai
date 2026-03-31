use windows::core::{Error, Result, HRESULT};
use windows::Win32::Security::Authorization::{
    AuthzAccessCheck, AUTHZ_ACCESS_CHECK_FLAGS, AUTHZ_CLIENT_CONTEXT_HANDLE,
};
use windows::Win32::Security::PSECURITY_DESCRIPTOR;

fn call_authz_access_check() -> HRESULT {
    unsafe {
        match AuthzAccessCheck(
            AUTHZ_ACCESS_CHECK_FLAGS(0),
            AUTHZ_CLIENT_CONTEXT_HANDLE(std::ptr::null_mut()),
            std::ptr::null(),
            None,
            PSECURITY_DESCRIPTOR(std::ptr::null_mut()),
            None,
            std::ptr::null_mut(),
            None,
        ) {
            Ok(()) => HRESULT(0),
            Err(e) => e.code(),
        }
    }
}
