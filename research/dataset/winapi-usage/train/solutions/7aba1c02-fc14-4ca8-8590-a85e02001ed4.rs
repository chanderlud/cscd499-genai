use windows::core::{Error, Result};
use windows::Win32::Foundation::{ERROR_SUCCESS, WIN32_ERROR};
use windows::Win32::System::SystemInformation::GetLocalTime;

fn call_get_local_time() -> WIN32_ERROR {
    // GetLocalTime doesn't return a Result, it returns SYSTEMTIME directly
    // and doesn't fail in a way that returns an error code
    let _time = unsafe { GetLocalTime() };
    ERROR_SUCCESS
}
