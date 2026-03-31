use windows::core::{Error, Result};
use windows::Win32::Storage::FileSystem::{CopyFileExW, COPYFILE_FLAGS};

fn call_copy_file_ex_w() -> Result<()> {
    // SAFETY: CopyFileExW is unsafe due to raw pointer parameters, but we pass None for them.
    unsafe {
        CopyFileExW(
            windows::core::w!("source.txt"),
            windows::core::w!("dest.txt"),
            None,
            None,
            None,
            COPYFILE_FLAGS(0),
        )
    }
}
