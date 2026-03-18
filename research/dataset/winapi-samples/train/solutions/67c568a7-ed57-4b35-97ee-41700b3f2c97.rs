use windows::core::{Result, HRESULT};
use windows::Win32::Foundation::INVALID_HANDLE_VALUE;
use windows::Win32::System::Threading::{
    GetCurrentProcess, GetPriorityClass, BELOW_NORMAL_PRIORITY_CLASS,
};

fn main() -> Result<()> {
    // SAFETY: GetCurrentProcess returns a valid handle or INVALID_HANDLE_VALUE.
    let process_handle = unsafe { GetCurrentProcess() };
    if process_handle == INVALID_HANDLE_VALUE {
        return Err(HRESULT::from_win32(std::io::Error::last_os_error().raw_os_error().unwrap_or(0) as u32).into());
    }

    // SAFETY: GetPriorityClass returns a non-zero value on success or 0 on failure.
    let priority_class = unsafe { GetPriorityClass(process_handle) };
    if priority_class == 0 {
        return Err(HRESULT::from_win32(std::io::Error::last_os_error().raw_os_error().unwrap_or(0) as u32).into());
    }

    // Compare using the raw value of the constant.
    if priority_class == BELOW_NORMAL_PRIORITY_CLASS.0 {
        println!("Process priority: Below Normal");
    } else {
        println!("Process priority: Normal or higher");
    }

    Ok(())
}
