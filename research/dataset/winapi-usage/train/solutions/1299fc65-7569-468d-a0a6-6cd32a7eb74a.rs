#![allow(unused_imports, dead_code)]
use windows::core::w;
use windows::core::HRESULT;
use windows::core::{Error, Result};
use windows::Win32::System::RestartManager::{RmAddFilter, RM_FILTER_ACTION};

fn call_rm_add_filter() -> HRESULT {
    let err = unsafe { RmAddFilter(0, w!(""), None, w!(""), RM_FILTER_ACTION(0)) };
    err.to_hresult()
}
