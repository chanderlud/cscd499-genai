use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::GetLastError;

#[allow(dead_code)]
fn call_get_last_error() -> HRESULT {
    unsafe { GetLastError().to_hresult() }
}
