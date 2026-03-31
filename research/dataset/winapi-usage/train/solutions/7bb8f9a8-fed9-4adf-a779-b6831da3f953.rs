use windows::core::{Error, Result};
use windows::Win32::Storage::Compression::{CloseCompressor, COMPRESSOR_HANDLE};

fn call_close_compressor() -> Result<Result<()>> {
    let handle = COMPRESSOR_HANDLE(std::ptr::null_mut());
    // SAFETY: Calling CloseCompressor with a null handle is safe; it will return an error.
    Ok(unsafe { CloseCompressor(handle) })
}
