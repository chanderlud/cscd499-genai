#![deny(warnings)]

use windows::Win32::Devices::Display::BRUSHOBJ_pvAllocRbrush;
use windows::Win32::Foundation::{GetLastError, WIN32_ERROR};

#[allow(dead_code)]
fn call_brushobj_pv_alloc_rbrush() -> WIN32_ERROR {
    let result = unsafe { BRUSHOBJ_pvAllocRbrush(std::ptr::null_mut(), 0) };
    if result.is_null() {
        unsafe { GetLastError() }
    } else {
        WIN32_ERROR(0)
    }
}
