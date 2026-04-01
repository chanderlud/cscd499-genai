use windows::core::Result;
use windows::Win32::Storage::Compression::{
    CreateDecompressor, COMPRESS_ALGORITHM, DECOMPRESSOR_HANDLE,
};

fn call_create_decompressor() -> windows::core::HRESULT {
    let mut handle: DECOMPRESSOR_HANDLE = DECOMPRESSOR_HANDLE(std::ptr::null_mut());
    let algorithm = COMPRESS_ALGORITHM(0);

    let result: Result<()> = unsafe { CreateDecompressor(algorithm, None, &mut handle) };

    result.map_or_else(|e| e.code(), |_| windows::core::HRESULT::from_win32(0))
}
