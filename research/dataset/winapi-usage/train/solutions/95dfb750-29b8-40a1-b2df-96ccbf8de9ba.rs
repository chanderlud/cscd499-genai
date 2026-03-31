use windows::core::{Error, Result, HRESULT};
use windows::Win32::Media::{timeGetSystemTime, MMTIME};

fn call_time_get_system_time() -> Result<u32> {
    let mut mmt = MMTIME::default();
    let result = unsafe { timeGetSystemTime(&mut mmt, std::mem::size_of::<MMTIME>() as u32) };
    if result != 0 {
        Err(Error::from_hresult(HRESULT::from_win32(result)))
    } else {
        Ok(result)
    }
}
