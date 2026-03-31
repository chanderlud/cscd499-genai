use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Storage::Compression::{CloseCompressor, COMPRESSOR_HANDLE};

fn call_close_compressor() -> WIN32_ERROR {
    match unsafe { CloseCompressor(COMPRESSOR_HANDLE(std::ptr::null_mut())) } {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
