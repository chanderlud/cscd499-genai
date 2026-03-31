#![deny(warnings)]

use windows::core::Result;
use windows::Win32::Storage::Compression::{Compress, COMPRESSOR_HANDLE};

fn call_compress() -> Result<()> {
    let mut compressed_size: usize = 0;
    // SAFETY: Compress is an unsafe Win32 API. We provide valid pointers and sizes.
    // A null handle and zero sizes are used as concrete placeholder values for this exercise.
    unsafe {
        Compress(
            COMPRESSOR_HANDLE(std::ptr::null_mut()),
            None,
            0,
            None,
            0,
            &mut compressed_size,
        )
    }
}

fn main() {
    let _ = call_compress();
}
