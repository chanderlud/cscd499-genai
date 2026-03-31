use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::WindowsProgramming::GetPrivateProfileIntW;

fn call_get_private_profile_int_w() -> WIN32_ERROR {
    unsafe {
        GetPrivateProfileIntW(
            windows::core::w!("App"),
            windows::core::w!("Key"),
            0,
            windows::core::w!("file.ini"),
        );
    }
    WIN32_ERROR(0)
}
