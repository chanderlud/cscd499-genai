use windows::core::{Error, Result};
use windows::Win32::Media::timeGetTime;

fn call_time_get_time() -> windows::core::Result<u32> {
    unsafe {
        let time = timeGetTime();
        Ok(time)
    }
}
