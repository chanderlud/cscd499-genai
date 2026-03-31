use windows::core::w;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Storage::FileSystem::{CopyFileExW, COPYFILE_FLAGS};

fn call_copy_file_ex_w() -> WIN32_ERROR {
    // SAFETY: CopyFileExW is an unsafe FFI function. We pass valid wide string literals
    // and None for optional callback/data/cancel parameters, which is safe and idiomatic.
    let result = unsafe {
        CopyFileExW(
            w!("source.txt"),
            w!("dest.txt"),
            None,
            None,
            None,
            COPYFILE_FLAGS(0),
        )
    };

    match result {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR::from_error(&e).unwrap_or(WIN32_ERROR(0)),
    }
}
