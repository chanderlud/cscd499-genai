use windows::core::PCWSTR;
use windows::core::{Error, Result};
use windows::Win32::Foundation::{HANDLE, WIN32_ERROR};
use windows::Win32::Storage::FileSystem::AddLogContainer;

fn call_add_log_container() -> WIN32_ERROR {
    let result = unsafe { AddLogContainer(HANDLE::default(), None, PCWSTR::null(), None) };
    match result {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
