use windows::core::{Error, Result, HRESULT};
use windows::Win32::Media::{timeGetDevCaps, TIMECAPS};

fn call_time_get_dev_caps() -> Result<u32> {
    let mut caps = TIMECAPS::default();
    let size = std::mem::size_of::<TIMECAPS>() as u32;
    // SAFETY: timeGetDevCaps expects a valid mutable pointer to a TIMECAPS struct and its size.
    let result = unsafe { timeGetDevCaps(&mut caps, size) };
    if result != 0 {
        Err(Error::from_hresult(HRESULT::from_win32(result)))?;
    }
    Ok(caps.wPeriodMin)
}
