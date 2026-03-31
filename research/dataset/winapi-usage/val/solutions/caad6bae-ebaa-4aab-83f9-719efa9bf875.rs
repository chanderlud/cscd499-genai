use windows::core::{Error, Result};
use windows::Win32::Security::Authorization::{AuthzAddSidsToContext, AUTHZ_CLIENT_CONTEXT_HANDLE};

fn call_authz_add_sids_to_context() -> windows::core::Result<windows::core::Result<()>> {
    let mut new_context = AUTHZ_CLIENT_CONTEXT_HANDLE(std::ptr::null_mut());
    let res = unsafe {
        AuthzAddSidsToContext(
            AUTHZ_CLIENT_CONTEXT_HANDLE(std::ptr::null_mut()),
            None,
            0,
            None,
            0,
            &mut new_context,
        )
    };
    Ok(res)
}
