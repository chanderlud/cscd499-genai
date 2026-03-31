use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::ProcessStatus::EnumPageFilesA;

fn call_enum_page_files_a() -> WIN32_ERROR {
    match unsafe { EnumPageFilesA(None, std::ptr::null_mut()) } {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
