use std::io;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows::core::PCWSTR;
use windows::core::{Error, Result};
use windows::Win32::Storage::FileSystem::CopyFileW;

fn to_wide_null(path: &Path) -> Vec<u16> {
    path.as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

pub fn copy_file_winapi(
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
    fail_if_exists: bool,
) -> io::Result<()> {
    let src_w = to_wide_null(src.as_ref());
    let dst_w = to_wide_null(dst.as_ref());

    unsafe {
        CopyFileW(
            PCWSTR(src_w.as_ptr()),
            PCWSTR(dst_w.as_ptr()),
            fail_if_exists,
        )
        .map_err(|e| io::Error::from_raw_os_error(e.code().0))?;
    }

    Ok(())
}
