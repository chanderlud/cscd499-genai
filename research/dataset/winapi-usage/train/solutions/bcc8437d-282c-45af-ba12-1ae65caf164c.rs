use windows::core::{Error, Result, HRESULT};
use windows::Win32::System::WindowsProgramming::GetPrivateProfileIntW;

fn call_get_private_profile_int_w() -> HRESULT {
    // SAFETY: Win32 API call with valid null-terminated wide string literals.
    unsafe {
        GetPrivateProfileIntW(
            windows::core::w!("App"),
            windows::core::w!("Key"),
            0,
            windows::core::w!("file.ini"),
        );
    }
    HRESULT::from_win32(0)
}
