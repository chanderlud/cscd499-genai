use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::RestartManager::{RmAddFilter, RM_FILTER_ACTION};

fn call_rm_add_filter() -> WIN32_ERROR {
    unsafe {
        RmAddFilter(
            0,
            windows::core::w!(""),
            None,
            windows::core::w!(""),
            RM_FILTER_ACTION(0),
        )
    }
}
