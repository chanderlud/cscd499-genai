use windows::core::{Error, Result};
use windows::Win32::System::SystemInformation::{GetSystemInfo, SYSTEM_INFO};

fn call_get_system_info() -> windows::Win32::Foundation::WIN32_ERROR {
    let mut info = SYSTEM_INFO::default();
    unsafe {
        GetSystemInfo(&mut info);
    }
    windows::Win32::Foundation::WIN32_ERROR(0)
}
