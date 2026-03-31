#![allow(unused_imports)]

use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Media::{timeGetSystemTime, MMTIME};

fn call_time_get_system_time() -> WIN32_ERROR {
    let mut mmt = MMTIME::default();
    // SAFETY: We pass a valid mutable pointer to MMTIME and its correct size.
    let result = unsafe { timeGetSystemTime(&mut mmt, std::mem::size_of::<MMTIME>() as u32) };
    WIN32_ERROR(result)
}
