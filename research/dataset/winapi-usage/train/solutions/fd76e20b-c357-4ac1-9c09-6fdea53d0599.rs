use windows::core::{Error, Result};
use windows::Win32::Storage::Compression::{
    CreateCompressor, COMPRESSOR_HANDLE, COMPRESS_ALGORITHM,
};

#[allow(dead_code)]
fn call_create_compressor() -> Result<windows::core::Result<()>> {
    let mut handle = COMPRESSOR_HANDLE(std::ptr::null_mut());
    // SAFETY: CreateCompressor requires a valid mutable pointer for the output handle.
    // We provide a properly initialized local variable and pass None for allocation routines.
    Ok(unsafe { CreateCompressor(COMPRESS_ALGORITHM(0), None, &mut handle) })
}
