use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Media::timeBeginPeriod;

#[allow(dead_code)]
fn call_time_begin_period() -> WIN32_ERROR {
    // SAFETY: timeBeginPeriod is a standard Win32 API; passing a valid period (1 ms) is safe.
    let result = unsafe { timeBeginPeriod(1) };
    WIN32_ERROR(result)
}
