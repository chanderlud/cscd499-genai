use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Error, Result};
use windows::Win32::System::WindowsProgramming::AddDelBackupEntryW;

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

fn call_add_del_backup_entry_w() -> Result<()> {
    let file_list = wide_null(OsStr::new("C:\\backup\\files.txt"));
    let backup_dir = wide_null(OsStr::new("C:\\backup"));
    let base_name = wide_null(OsStr::new("backup"));

    unsafe {
        AddDelBackupEntryW(
            windows::core::PCWSTR::from_raw(file_list.as_ptr()),
            windows::core::PCWSTR::from_raw(backup_dir.as_ptr()),
            windows::core::PCWSTR::from_raw(base_name.as_ptr()),
            0,
        )
    }
}
