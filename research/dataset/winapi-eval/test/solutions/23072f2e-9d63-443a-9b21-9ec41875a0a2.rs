use std::ffi::OsStr;
use std::iter;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use windows::core::{Result, PCWSTR};
use windows::Win32::Storage::FileSystem::MoveFileW;

fn to_wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(iter::once(0)).collect()
}

pub fn rename_file(from: &Path, to: &Path) -> Result<()> {
    let from_w = to_wide_null(from.as_os_str());
    let to_w = to_wide_null(to.as_os_str());

    unsafe { MoveFileW(PCWSTR(from_w.as_ptr()), PCWSTR(to_w.as_ptr())) }
}
