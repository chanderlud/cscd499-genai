use windows::core::BOOL;
use windows::core::{Error, Result};
use windows::Win32::NetworkManagement::IpHelper::CancelIPChangeNotify;
use windows::Win32::System::IO::OVERLAPPED;

fn call_cancel_ip_change_notify() -> Result<BOOL> {
    let overlapped = OVERLAPPED::default();
    // SAFETY: Passing a valid pointer to an OVERLAPPED structure as required by the API.
    let result = unsafe { CancelIPChangeNotify(&overlapped) };
    if result.0 == 0 {
        Err(Error::from_thread())
    } else {
        Ok(result)
    }
}
