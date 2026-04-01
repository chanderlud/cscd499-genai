use windows::core::Result;
use windows::Win32::Security::Authorization::{
    AuthzCachedAccessCheck, AUTHZ_ACCESS_CHECK_RESULTS_HANDLE, AUTHZ_ACCESS_REPLY,
    AUTHZ_ACCESS_REQUEST, AUTHZ_AUDIT_EVENT_HANDLE,
};
use windows::Win32::Security::PSID;

fn call_authz_cached_access_check() -> Result<()> {
    // Create concrete parameter values for the API call
    let flags = 0u32;
    let haccesscheckresults = AUTHZ_ACCESS_CHECK_RESULTS_HANDLE(core::ptr::null_mut());
    let prequest = AUTHZ_ACCESS_REQUEST {
        DesiredAccess: 0,
        PrincipalSelfSid: PSID(core::ptr::null_mut()),
        ObjectTypeList: core::ptr::null_mut(),
        ObjectTypeListLength: 0,
        OptionalArguments: core::ptr::null_mut(),
    };
    let hauditevent: Option<AUTHZ_AUDIT_EVENT_HANDLE> = None;
    let mut preply = AUTHZ_ACCESS_REPLY {
        ResultListLength: 0,
        GrantedAccessMask: core::ptr::null_mut(),
        SaclEvaluationResults: core::ptr::null_mut(),
        Error: core::ptr::null_mut(),
    };

    // Call AuthzCachedAccessCheck and convert Result to WIN32_ERROR
    unsafe {
        AuthzCachedAccessCheck(
            flags,
            haccesscheckresults,
            &prequest,
            hauditevent,
            &mut preply,
        )?;
    }

    Ok(())
}
