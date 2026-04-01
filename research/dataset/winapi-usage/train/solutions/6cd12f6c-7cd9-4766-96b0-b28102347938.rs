use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::{HANDLE, S_OK};
use windows::Win32::System::IO::{GetOverlappedResult, OVERLAPPED};

fn call_get_overlapped_result() -> HRESULT {
    let hfile = HANDLE(std::ptr::null_mut());
    let mut overlapped = OVERLAPPED::default();
    let mut bytes_transferred: u32 = 0;

    let result: Result<()> =
        unsafe { GetOverlappedResult(hfile, &overlapped, &mut bytes_transferred, true) };

    result.map(|_| S_OK).unwrap_or_else(|e| e.code())
}
