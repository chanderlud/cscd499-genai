use windows::core::{Error, Result};
use windows::Win32::Security::Authentication::Identity::CompleteAuthToken;

fn call_complete_auth_token() -> windows::Win32::Foundation::WIN32_ERROR {
    // SAFETY: Passing null pointers is safe for this API call exercise;
    // the function will return an error which we handle.
    let result = unsafe { CompleteAuthToken(std::ptr::null(), std::ptr::null()) };

    match result {
        Ok(()) => windows::Win32::Foundation::WIN32_ERROR(0),
        Err(e) => windows::Win32::Foundation::WIN32_ERROR(e.code().0 as u32),
    }
}
