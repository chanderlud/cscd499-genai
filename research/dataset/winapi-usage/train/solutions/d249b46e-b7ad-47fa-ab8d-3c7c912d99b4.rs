use windows::core::{w, Error, HRESULT};
use windows::Win32::Foundation::S_OK;
use windows::Win32::UI::ColorSystem::AssociateColorProfileWithDeviceW;

fn call_associate_color_profile_with_device_w() -> HRESULT {
    let result = unsafe { AssociateColorProfileWithDeviceW(w!(""), w!(""), w!("")) };
    if result.as_bool() {
        S_OK
    } else {
        Error::from_thread().code()
    }
}
