#![deny(warnings)]

use windows::core::HRESULT;
use windows::Win32::Storage::Compression::{CloseDecompressor, DECOMPRESSOR_HANDLE};

#[allow(dead_code)]
fn call_close_decompressor() -> HRESULT {
    // SAFETY: Passing a null handle is safe for this exercise; the API will return an error.
    unsafe {
        CloseDecompressor(DECOMPRESSOR_HANDLE(std::ptr::null_mut()))
            .map(|_| HRESULT(0))
            .unwrap_or_else(|e| e.code())
    }
}
