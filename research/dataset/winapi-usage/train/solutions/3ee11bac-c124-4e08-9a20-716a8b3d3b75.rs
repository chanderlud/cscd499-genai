use windows::core::{Error, Result, BOOL};
use windows::Win32::UI::ColorSystem::AssociateColorProfileWithDeviceW;

#[allow(dead_code)]
fn call_associate_color_profile_with_device_w() -> Result<BOOL> {
    let result = unsafe {
        AssociateColorProfileWithDeviceW(
            windows::core::w!(""),
            windows::core::w!("sRGB Color Space Profile.icm"),
            windows::core::w!("DISPLAY1"),
        )
    };
    if result == false {
        Err(Error::from_thread())
    } else {
        Ok(result)
    }
}
