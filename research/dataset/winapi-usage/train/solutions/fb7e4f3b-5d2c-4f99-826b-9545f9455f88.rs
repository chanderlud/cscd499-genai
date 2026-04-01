use windows::Win32::Foundation::{GetLastError, ERROR_SUCCESS, WIN32_ERROR};
use windows::Win32::Storage::Compression::{
    CreateDecompressor, COMPRESS_ALGORITHM, DECOMPRESSOR_HANDLE,
};

fn call_create_decompressor() -> windows::Win32::Foundation::WIN32_ERROR {
    let mut handle: DECOMPRESSOR_HANDLE = DECOMPRESSOR_HANDLE(std::ptr::null_mut());

    unsafe {
        match CreateDecompressor(COMPRESS_ALGORITHM(0), None, &mut handle) {
            Ok(_) => ERROR_SUCCESS,
            Err(_) => {
                let win32_code = unsafe { GetLastError().0 };
                WIN32_ERROR(win32_code)
            }
        }
    }
}
