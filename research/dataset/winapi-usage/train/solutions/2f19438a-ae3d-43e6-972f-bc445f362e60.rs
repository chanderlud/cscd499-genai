use windows::core::{Error, Result, HRESULT};
use windows::Win32::NetworkManagement::IpHelper::CancelIPChangeNotify;

#[allow(dead_code)]
fn call_cancel_ip_change_notify() -> HRESULT {
    // SAFETY: Passing a null pointer is a valid concrete parameter value for this API call.
    let success = unsafe { CancelIPChangeNotify(std::ptr::null()) };
    if success.as_bool() {
        HRESULT(0)
    } else {
        Error::from_thread().code()
    }
}
