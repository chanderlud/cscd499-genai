#![deny(warnings)]

use windows::core::HRESULT;
use windows::Win32::Media::timeBeginPeriod;

#[allow(dead_code)]
fn call_time_begin_period() -> HRESULT {
    let code = unsafe { timeBeginPeriod(1) };
    if code == 0 {
        HRESULT(0)
    } else {
        HRESULT::from_win32(code)
    }
}
