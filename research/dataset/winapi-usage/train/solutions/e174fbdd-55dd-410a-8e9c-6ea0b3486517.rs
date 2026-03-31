use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Media::{timeGetDevCaps, TIMECAPS};

fn call_time_get_dev_caps() -> WIN32_ERROR {
    let mut tc = TIMECAPS::default();
    let cbtc = std::mem::size_of::<TIMECAPS>() as u32;
    // SAFETY: `tc` is a valid mutable reference, and `cbtc` correctly specifies its size.
    let res = unsafe { timeGetDevCaps(&mut tc, cbtc) };
    WIN32_ERROR(res)
}
