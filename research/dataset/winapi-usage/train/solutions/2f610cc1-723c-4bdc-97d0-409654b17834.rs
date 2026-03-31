use windows::core::{Error, Result};
use windows::Win32::Storage::Compression::{CloseCompressor, COMPRESSOR_HANDLE};

fn call_close_compressor() -> windows::core::HRESULT {
    // SAFETY: Calling CloseCompressor with a null handle is safe for this exercise.
    match unsafe { CloseCompressor(COMPRESSOR_HANDLE(std::ptr::null_mut())) } {
        Ok(()) => windows::core::HRESULT(0),
        Err(e) => e.code(),
    }
}
