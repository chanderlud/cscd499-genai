use windows::core::{Error, Result};
use windows::Win32::Foundation::{GetLastError, HANDLE, HMODULE, WIN32_ERROR};
use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;

fn call_get_module_file_name_ex_w() -> windows::Win32::Foundation::WIN32_ERROR {
    let mut filename = [0u16; 260];

    let result = unsafe {
        GetModuleFileNameExW(
            Some(HANDLE::default()),
            Some(HMODULE::default()),
            &mut filename,
        )
    };

    if result == 0 {
        let error_code = unsafe { GetLastError() };
        WIN32_ERROR(error_code.0)
    } else {
        WIN32_ERROR(0)
    }
}
