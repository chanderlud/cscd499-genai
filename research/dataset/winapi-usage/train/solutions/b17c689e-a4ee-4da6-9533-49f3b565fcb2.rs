use windows::core::{w, Error, Result, HRESULT};
use windows::Win32::Storage::FileSystem::{CopyFileExW, COPYFILE_FLAGS};

#[allow(dead_code)]
fn call_copy_file_ex_w() -> HRESULT {
    unsafe {
        CopyFileExW(
            w!("source.txt"),
            w!("dest.txt"),
            None,
            None,
            None,
            COPYFILE_FLAGS(0),
        )
        .map(|_| HRESULT(0))
        .unwrap_or_else(|e| e.code())
    }
}
