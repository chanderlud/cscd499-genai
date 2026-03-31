use windows::core::{Error, Result};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::IO::CancelIo;

fn call_cancel_io() -> Result<Result<()>> {
    let hfile = HANDLE(std::ptr::null_mut());
    // SAFETY: CancelIo is an unsafe Win32 API. Passing a null handle is valid for
    // demonstration purposes; the API will safely return an error Result.
    Ok(unsafe { CancelIo(hfile) })
}
