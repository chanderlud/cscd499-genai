use windows::core::HRESULT;
use windows::core::{Error, Result};
use windows::Win32::Foundation::S_OK;
use windows::Win32::Security::Authorization::{
    AuthzCachedAccessCheck, AUTHZ_ACCESS_CHECK_RESULTS_HANDLE, AUTHZ_ACCESS_REPLY,
    AUTHZ_ACCESS_REQUEST, AUTHZ_AUDIT_EVENT_HANDLE,
};

fn call_authz_cached_access_check() -> HRESULT {
    unsafe {
        let flags = 0u32;
        let haccesscheckresults = AUTHZ_ACCESS_CHECK_RESULTS_HANDLE(std::ptr::null_mut());
        let prequest = std::ptr::null::<AUTHZ_ACCESS_REQUEST>();
        let hauditevent: Option<AUTHZ_AUDIT_EVENT_HANDLE> = None;
        let mut preply = AUTHZ_ACCESS_REPLY {
            ResultListLength: 0,
            GrantedAccessMask: std::ptr::null_mut(),
            SaclEvaluationResults: std::ptr::null_mut(),
            Error: std::ptr::null_mut(),
        };

        match AuthzCachedAccessCheck(
            flags,
            haccesscheckresults,
            prequest,
            hauditevent,
            &mut preply,
        ) {
            Ok(_) => S_OK,
            Err(e) => e.code(),
        }
    }
}
