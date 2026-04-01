use windows::core::{Error, Result};
use windows::Win32::Foundation::{HANDLE, HMODULE};
use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;

fn call_get_module_file_name_ex_w() -> Result<u32> {
    let mut buffer = [0u16; 260];

    // Call GetModuleFileNameExW with None for current process and module
    let result = unsafe { GetModuleFileNameExW(None, None, &mut buffer) };

    // Check for error (0 indicates failure)
    if result == 0 {
        return Err(Error::from_thread());
    }

    Ok(result)
}
