use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::NetworkManagement::IpHelper::CancelMibChangeNotify2;

fn call_cancel_mib_change_notify2() -> HRESULT {
    let result = unsafe { CancelMibChangeNotify2(HANDLE::default()) };
    result.to_hresult()
}
