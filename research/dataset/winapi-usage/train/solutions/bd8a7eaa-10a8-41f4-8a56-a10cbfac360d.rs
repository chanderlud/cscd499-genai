use windows::core::HRESULT;
use windows::Win32::Foundation::S_OK;
use windows::Win32::Storage::Compression::{
    CreateCompressor, COMPRESSOR_HANDLE, COMPRESS_ALGORITHM,
};

fn call_create_compressor() -> HRESULT {
    let mut handle = COMPRESSOR_HANDLE(std::ptr::null_mut());
    // SAFETY: CreateCompressor writes to the provided pointer, which is valid and properly initialized.
    unsafe {
        match CreateCompressor(COMPRESS_ALGORITHM(2), None, &mut handle) {
            Ok(()) => S_OK,
            Err(e) => e.code(),
        }
    }
}
