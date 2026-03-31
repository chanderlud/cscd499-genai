use windows::core::{Error, Result, BOOL};
use windows::Win32::UI::ColorSystem::AssociateColorProfileWithDeviceA;

fn call_associate_color_profile_with_device_a() -> Result<BOOL> {
    let machine_name = windows::core::PCSTR::null();
    let profile_name = windows::core::PCSTR::null();
    let device_name = windows::core::PCSTR::null();

    let result =
        unsafe { AssociateColorProfileWithDeviceA(machine_name, profile_name, device_name) };
    if result == false {
        Err(Error::from_thread())
    } else {
        Ok(result)
    }
}
