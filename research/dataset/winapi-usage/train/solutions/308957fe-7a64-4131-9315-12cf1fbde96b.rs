use windows::core::{w, Error, Result};
use windows::Win32::Foundation::{ERROR_SUCCESS, WIN32_ERROR};
use windows::Win32::System::RestartManager::{RmAddFilter, RM_FILTER_ACTION};

fn call_rm_add_filter() -> Result<WIN32_ERROR> {
    let result = unsafe { RmAddFilter(1, w!(""), None, w!(""), RM_FILTER_ACTION(0)) };
    if result == ERROR_SUCCESS {
        Ok(result)
    } else {
        Err(Error::from_hresult(result.to_hresult()))
    }
}
