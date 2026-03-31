#![allow(dead_code)]

use windows::core::{Error, Result, BOOL, PCSTR};
use windows::Win32::System::ProcessStatus::{
    EnumPageFilesA, ENUM_PAGE_FILE_INFORMATION, PENUM_PAGE_FILE_CALLBACKA,
};

unsafe extern "system" fn page_file_callback(
    _context: *mut core::ffi::c_void,
    _info: *mut ENUM_PAGE_FILE_INFORMATION,
    _filename: PCSTR,
) -> BOOL {
    BOOL(1)
}

fn call_enum_page_files_a() -> Result<Result<()>> {
    let callback: PENUM_PAGE_FILE_CALLBACKA = Some(page_file_callback);
    // SAFETY: The callback matches the expected signature and safely handles enumeration.
    // pcontext is null as no user context is required.
    Ok(unsafe { EnumPageFilesA(callback, std::ptr::null_mut()) })
}
