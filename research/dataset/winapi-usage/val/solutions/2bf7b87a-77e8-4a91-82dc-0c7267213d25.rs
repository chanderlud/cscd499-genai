use windows::core::HRESULT;
use windows::Win32::Security::Authorization::{AuthzAddSidsToContext, AUTHZ_CLIENT_CONTEXT_HANDLE};

fn call_authz_add_sids_to_context() -> windows::core::HRESULT {
    let mut new_context = AUTHZ_CLIENT_CONTEXT_HANDLE(std::ptr::null_mut());
    unsafe {
        AuthzAddSidsToContext(
            AUTHZ_CLIENT_CONTEXT_HANDLE(std::ptr::null_mut()),
            None,
            0,
            None,
            0,
            &mut new_context,
        )
    }
    .map(|_| HRESULT(0))
    .unwrap_or_else(|e| e.code())
}
