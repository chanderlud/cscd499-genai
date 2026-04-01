use windows::core::{Error, Result};
use windows::Win32::Foundation::S_OK;
use windows::Win32::Media::timeGetTime;

fn call_time_get_time() -> windows::core::HRESULT {
    unsafe { timeGetTime() };
    S_OK
}
