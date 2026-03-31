use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::{HANDLE, WIN32_ERROR};
use windows::Win32::NetworkManagement::IpHelper::CancelMibChangeNotify2;

#[allow(dead_code)]
fn call_cancel_mib_change_notify2() -> Result<WIN32_ERROR> {
    // SAFETY: Calling an unsafe Win32 API with a concrete null handle value.
    let result = unsafe { CancelMibChangeNotify2(HANDLE::default()) };
    if result.0 != 0 {
        return Err(Error::from_hresult(HRESULT::from_win32(result.0)));
    }
    Ok(result)
}
