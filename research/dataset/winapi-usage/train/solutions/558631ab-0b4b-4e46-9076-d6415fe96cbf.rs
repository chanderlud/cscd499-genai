use windows::core::{Error, Result, HRESULT};
use windows::Win32::System::WindowsProgramming::AddDelBackupEntryA;

fn call_add_del_backup_entry_a() -> HRESULT {
    unsafe {
        AddDelBackupEntryA(
            windows::core::PCSTR::null(),
            windows::core::PCSTR::null(),
            windows::core::PCSTR::null(),
            0,
        )
        .map(|_| HRESULT::default())
        .unwrap_or_else(|e| e.code())
    }
}
