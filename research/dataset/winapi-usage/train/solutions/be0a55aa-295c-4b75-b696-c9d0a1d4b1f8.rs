use windows::core::PCWSTR;
use windows::Win32::Foundation::{INVALID_HANDLE_VALUE, WIN32_ERROR};
use windows::Win32::System::Memory::{CreateFileMappingW, PAGE_READWRITE};

fn call_create_file_mapping_w() -> WIN32_ERROR {
    unsafe {
        match CreateFileMappingW(
            INVALID_HANDLE_VALUE,
            None,
            PAGE_READWRITE,
            0,
            1024,
            PCWSTR::null(),
        ) {
            Ok(_) => WIN32_ERROR(0),
            Err(e) => WIN32_ERROR(e.code().0 as u32),
        }
    }
}
