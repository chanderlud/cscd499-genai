use windows::Win32::Storage::Compression::{
    CreateDecompressor, COMPRESS_ALGORITHM, DECOMPRESSOR_HANDLE,
};

fn call_create_decompressor() -> windows::core::Result<()> {
    // Create a mutable variable to hold the decompressor handle
    let mut handle: DECOMPRESSOR_HANDLE = DECOMPRESSOR_HANDLE(std::ptr::null_mut());

    // Call CreateDecompressor with concrete parameters
    // Using COMPRESS_ALGORITHM_MSZIP (value 2) as a reasonable default algorithm
    // Passing None for allocation routines to use system defaults
    unsafe { CreateDecompressor(COMPRESS_ALGORITHM(2), None, &mut handle) }
}
