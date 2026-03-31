#![allow(dead_code)]

use windows::core::Error;
use windows::core::PCSTR;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::UI::ColorSystem::AssociateColorProfileWithDeviceA;

fn call_associate_color_profile_with_device_a() -> WIN32_ERROR {
    let result =
        unsafe { AssociateColorProfileWithDeviceA(PCSTR::null(), PCSTR::null(), PCSTR::null()) };
    if result.as_bool() {
        WIN32_ERROR(0)
    } else {
        let err = Error::from_thread();
        WIN32_ERROR(err.code().0 as u32)
    }
}
