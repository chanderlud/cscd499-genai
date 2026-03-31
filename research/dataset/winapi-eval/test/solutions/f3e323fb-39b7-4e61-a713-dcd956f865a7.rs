use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows::core::{Result, PCWSTR};
use windows::Win32::Storage::FileSystem::DeleteFileW;

pub fn delete_file(path: &Path) -> Result<()> {
    let wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe { DeleteFileW(PCWSTR(wide.as_ptr())) }
}
