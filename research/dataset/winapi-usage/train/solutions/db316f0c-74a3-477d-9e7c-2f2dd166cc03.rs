use windows::core::{Error, Result};
use windows::Win32::System::SystemInformation::{GetSystemInfo, SYSTEM_INFO};

fn call_get_system_info() -> windows::core::HRESULT {
    let mut info = SYSTEM_INFO::default();
    // SAFETY: GetSystemInfo expects a valid mutable pointer to a SYSTEM_INFO struct, which we provide.
    unsafe {
        GetSystemInfo(&mut info);
    }
    windows::core::HRESULT::from_win32(0)
}
