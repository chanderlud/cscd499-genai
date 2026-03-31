use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE};
use windows::Win32::System::Memory::{CreateFileMappingW, PAGE_READWRITE};

fn call_create_file_mapping_w() -> Result<HANDLE> {
    let handle = unsafe {
        CreateFileMappingW(
            INVALID_HANDLE_VALUE,
            None,
            PAGE_READWRITE,
            0,
            1024 * 1024,
            PCWSTR::null(),
        )
    }?;
    Ok(handle)
}
