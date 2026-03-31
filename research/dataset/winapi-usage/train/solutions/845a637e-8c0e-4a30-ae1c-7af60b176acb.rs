use windows::core::{Error, Result, HRESULT};
use windows::Win32::Media::timeEndPeriod;

fn call_time_end_period() -> HRESULT {
    unsafe { HRESULT::from_win32(timeEndPeriod(1)) }
}
