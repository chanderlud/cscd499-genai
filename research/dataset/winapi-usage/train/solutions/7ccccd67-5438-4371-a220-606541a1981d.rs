use windows::core::{Error, Result, PCSTR};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::WindowsProgramming::AddDelBackupEntryA;

fn call_add_del_backup_entry_a() -> WIN32_ERROR {
    let result = unsafe { AddDelBackupEntryA(PCSTR::null(), PCSTR::null(), PCSTR::null(), 0) };
    match result {
        Ok(()) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
