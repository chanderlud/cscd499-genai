use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Storage::Compression::{Compress, COMPRESSOR_HANDLE};

fn call_compress() -> WIN32_ERROR {
    let mut compressed_size: usize = 0;
    // SAFETY: Calling Compress with null/dummy handles and buffers to demonstrate the API call.
    // The function will return an error, which we handle and convert to WIN32_ERROR.
    let result = unsafe {
        Compress(
            COMPRESSOR_HANDLE(std::ptr::null_mut()),
            None,
            0,
            None,
            0,
            &mut compressed_size,
        )
    };

    match result {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
