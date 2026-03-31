use windows::core::{Error, Result};
use windows::Win32::Networking::WinHttp::WinHttpCloseHandle;

#[allow(clippy::result_result)]
fn call_win_http_close_handle() -> Result<Result<()>> {
    let handle = std::ptr::null_mut::<core::ffi::c_void>();
    // SAFETY: Passing a null handle is safe; the API handles it and returns an appropriate Result.
    Ok(unsafe { WinHttpCloseHandle(handle) })
}
