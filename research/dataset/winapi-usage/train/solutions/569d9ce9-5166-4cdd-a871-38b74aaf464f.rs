use windows::core::{Error, Result};
use windows::Win32::Media::{timeGetDevCaps, TIMECAPS};

fn call_time_get_dev_caps() -> windows::core::HRESULT {
    let mut caps = TIMECAPS::default();
    let result = unsafe { timeGetDevCaps(&mut caps, std::mem::size_of::<TIMECAPS>() as u32) };
    windows::core::HRESULT::from_win32(result)
}
