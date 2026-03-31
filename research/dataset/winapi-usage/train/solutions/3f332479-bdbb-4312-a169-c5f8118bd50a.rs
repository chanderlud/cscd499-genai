use windows::core::{Error, Result};
use windows::Win32::Media::timeBeginPeriod;

fn call_time_begin_period() -> Result<u32> {
    // SAFETY: timeBeginPeriod is a standard Win32 API safe to call with a valid period value.
    let res = unsafe { timeBeginPeriod(1) };
    if res == 0 {
        Ok(res)
    } else {
        Err(Error::from_thread())
    }
}
