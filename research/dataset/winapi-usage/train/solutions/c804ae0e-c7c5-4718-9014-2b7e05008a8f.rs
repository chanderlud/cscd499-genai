use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Media::timeEndPeriod;

fn call_time_end_period() -> WIN32_ERROR {
    // SAFETY: timeEndPeriod is a standard Win32 API; 1 is a valid timer resolution.
    let res = unsafe { timeEndPeriod(1) };
    WIN32_ERROR(res)
}
