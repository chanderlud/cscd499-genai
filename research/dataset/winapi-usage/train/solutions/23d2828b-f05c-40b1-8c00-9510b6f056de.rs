use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::ProcessStatus::EnumDeviceDrivers;

fn call_enum_device_drivers() -> WIN32_ERROR {
    // SAFETY: Passing null pointers and 0 for cb is safe for querying required size.
    let result: Result<()> =
        unsafe { EnumDeviceDrivers(std::ptr::null_mut(), 0, std::ptr::null_mut()) };

    match result {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
