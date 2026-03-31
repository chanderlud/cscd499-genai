use windows::core::{Error, Result, HRESULT};
use windows::Win32::Security::Authentication::Identity::CompleteAuthToken;

fn call_complete_auth_token() -> HRESULT {
    unsafe {
        CompleteAuthToken(std::ptr::null(), std::ptr::null())
            .map(|_| HRESULT(0))
            .unwrap_or_else(|e| e.code())
    }
}
