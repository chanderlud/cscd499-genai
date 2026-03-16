use windows::core::{Error, HRESULT};
use windows::Win32::Foundation::ERROR_ACCESS_DENIED;

pub fn is_access_denied(err: &Error) -> bool {
    err.code() == HRESULT::from_win32(ERROR_ACCESS_DENIED.0)
}
