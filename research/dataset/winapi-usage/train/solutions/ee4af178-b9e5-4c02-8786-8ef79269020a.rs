use windows::core::{Error, Result};
use windows::Win32::System::WindowsProgramming::AddDelBackupEntryA;

fn call_add_del_backup_entry_a() -> Result<()> {
    unsafe {
        AddDelBackupEntryA(
            windows::core::s!("file1.txt"),
            windows::core::s!("C:\\backup"),
            windows::core::s!("backup"),
            0,
        )
    }
}
