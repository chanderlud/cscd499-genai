use windows::core::{Error, Result, HRESULT};
use windows::Win32::Storage::Compression::{Compress, COMPRESSOR_HANDLE};

fn call_compress() -> HRESULT {
    unsafe {
        Compress(
            COMPRESSOR_HANDLE(std::ptr::null_mut()),
            None,
            0,
            None,
            0,
            std::ptr::null_mut(),
        )
        .map(|_| HRESULT(0))
        .unwrap_or_else(|e| e.code())
    }
}
