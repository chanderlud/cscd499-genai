#![deny(warnings)]

use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Security::Authorization::{AuthzAddSidsToContext, AUTHZ_CLIENT_CONTEXT_HANDLE};

#[allow(dead_code)]
fn call_authz_add_sids_to_context() -> WIN32_ERROR {
    let mut new_context = AUTHZ_CLIENT_CONTEXT_HANDLE(std::ptr::null_mut());
    // SAFETY: We pass null/zero parameters as concrete values for this exercise.
    // The API will return an error, which we safely capture and convert.
    unsafe {
        match AuthzAddSidsToContext(
            AUTHZ_CLIENT_CONTEXT_HANDLE(std::ptr::null_mut()),
            None,
            0,
            None,
            0,
            &mut new_context,
        ) {
            Ok(()) => WIN32_ERROR(0),
            Err(e) => WIN32_ERROR::from_error(&e).unwrap_or(WIN32_ERROR(0)),
        }
    }
}
