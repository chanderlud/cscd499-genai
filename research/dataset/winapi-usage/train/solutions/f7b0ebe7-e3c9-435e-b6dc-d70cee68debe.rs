use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Storage::Compression::{CloseDecompressor, DECOMPRESSOR_HANDLE};

fn call_close_decompressor() -> WIN32_ERROR {
    match unsafe { CloseDecompressor(DECOMPRESSOR_HANDLE(std::ptr::null_mut())) } {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR::from_error(&e).unwrap_or_default(),
    }
}
