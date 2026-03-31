use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::NetworkManagement::IpHelper::CancelIPChangeNotify;
use windows::Win32::System::IO::OVERLAPPED;

fn call_cancel_ip_change_notify() -> WIN32_ERROR {
    unsafe {
        if CancelIPChangeNotify(std::ptr::null::<OVERLAPPED>()).as_bool() {
            WIN32_ERROR(0)
        } else {
            WIN32_ERROR(Error::from_thread().code().0 as u32)
        }
    }
}
