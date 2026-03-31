#![deny(warnings)]

use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::UI::ColorSystem::AssociateColorProfileWithDeviceW;

#[allow(dead_code)]
fn call_associate_color_profile_with_device_w() -> WIN32_ERROR {
    match unsafe { AssociateColorProfileWithDeviceW(None, None, None).ok() } {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
