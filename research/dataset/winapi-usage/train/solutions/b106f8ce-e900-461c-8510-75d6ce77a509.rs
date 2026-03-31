use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Power::CanUserWritePwrScheme;

fn call_can_user_write_pwr_scheme() -> WIN32_ERROR {
    // SAFETY: CanUserWritePwrScheme and GetLastError are standard Win32 APIs with no special safety contracts.
    if unsafe { CanUserWritePwrScheme() } {
        WIN32_ERROR(0)
    } else {
        unsafe { windows::Win32::Foundation::GetLastError() }
    }
}
