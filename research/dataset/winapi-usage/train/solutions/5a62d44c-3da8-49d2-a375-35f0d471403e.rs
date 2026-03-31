#[allow(unused_imports)]
use windows::core::{Error, Result};
use windows::core::{HRESULT, PCSTR};
use windows::Win32::UI::ColorSystem::AssociateColorProfileWithDeviceA;

fn call_associate_color_profile_with_device_a() -> HRESULT {
    // SAFETY: Passing null parameters to the Win32 API is safe; it will return FALSE on failure.
    let success =
        unsafe { AssociateColorProfileWithDeviceA(PCSTR::null(), PCSTR::null(), PCSTR::null()) };
    if success.as_bool() {
        HRESULT::from_win32(0)
    } else {
        Error::from_thread().code()
    }
}
