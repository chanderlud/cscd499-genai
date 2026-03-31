use windows::core::{Error, Result};
use windows::Win32::Foundation::{CloseHandle, HANDLE};

fn call_close_handle() -> Result<()> {
    unsafe { CloseHandle(HANDLE::default()) }
}
