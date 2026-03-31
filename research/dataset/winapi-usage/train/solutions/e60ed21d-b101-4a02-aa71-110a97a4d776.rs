use windows::core::{Error, Result};
use windows::Win32::Storage::Compression::{CloseDecompressor, DECOMPRESSOR_HANDLE};

fn call_close_decompressor() -> Result<Result<()>> {
    let handle = DECOMPRESSOR_HANDLE(std::ptr::null_mut());
    // SAFETY: Passing a null handle is safe; the API will return an error rather than UB.
    let res = unsafe { CloseDecompressor(handle) };
    Ok(res)
}
