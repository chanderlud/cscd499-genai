use windows::core::{Error, Result};
use windows::Win32::System::WindowsProgramming::GetPrivateProfileIntW;

fn call_get_private_profile_int_w() -> Result<i32> {
    // SAFETY: Passing valid null-terminated wide string literals is safe for this API.
    let value = unsafe {
        GetPrivateProfileIntW(
            windows::core::w!("App"),
            windows::core::w!("Key"),
            0,
            windows::core::w!("file.ini"),
        )
    };
    Ok(value)
}
