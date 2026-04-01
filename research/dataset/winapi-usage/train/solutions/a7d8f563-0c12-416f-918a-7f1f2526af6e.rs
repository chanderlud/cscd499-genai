use windows::core::{Error, Result};
use windows::Win32::Foundation::SYSTEMTIME;
use windows::Win32::System::SystemInformation::GetLocalTime;

fn call_get_local_time() -> Result<SYSTEMTIME> {
    unsafe { Ok(GetLocalTime()) }
}
