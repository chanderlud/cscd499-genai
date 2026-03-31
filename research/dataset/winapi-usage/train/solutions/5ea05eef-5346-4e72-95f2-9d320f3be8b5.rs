use windows::core::HRESULT;
use windows::Win32::System::ProcessStatus::EnumDeviceDrivers;

fn call_enum_device_drivers() -> HRESULT {
    let mut needed = 0u32;
    unsafe {
        match EnumDeviceDrivers(std::ptr::null_mut(), 0, &mut needed) {
            Ok(()) => HRESULT(0),
            Err(e) => e.code(),
        }
    }
}
