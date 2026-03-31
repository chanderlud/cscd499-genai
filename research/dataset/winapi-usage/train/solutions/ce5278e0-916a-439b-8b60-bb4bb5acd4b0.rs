use windows::core::{Error, Result};
use windows::Win32::Media::timeEndPeriod;

fn call_time_end_period() -> Result<u32> {
    // SAFETY: timeEndPeriod is a standard Win32 API; passing 1 ms is a valid resolution.
    let res = unsafe { timeEndPeriod(1) };
    if res != 0 {
        return Err(Error::from_thread());
    }
    Ok(res)
}
