use windows::core::{Error, Result};
use windows::Win32::Foundation::{HANDLE, WIN32_ERROR};
use windows::Win32::System::IO::{GetOverlappedResult, OVERLAPPED};

fn call_get_overlapped_result() -> Result<WIN32_ERROR> {
    let hfile = HANDLE::default();
    let mut overlapped = OVERLAPPED::default();
    let mut bytes_transferred: u32 = 0;

    match unsafe { GetOverlappedResult(hfile, &overlapped, &mut bytes_transferred, true) } {
        Ok(_) => Ok(WIN32_ERROR(0)),
        Err(e) => Err(e),
    }
}
