use windows::core::{w, Error, Result, HRESULT, PCWSTR};
use windows::Win32::System::WindowsProgramming::AddDelBackupEntryW;

fn call_add_del_backup_entry_w() -> Result<HRESULT> {
    unsafe {
        AddDelBackupEntryW(
            PCWSTR::from_raw(w!("C:\\Test\\file.txt").as_ptr()),
            PCWSTR::from_raw(w!("C:\\Backup").as_ptr()),
            PCWSTR::from_raw(w!("backup").as_ptr()),
            0,
        )?;
        Ok(HRESULT::from_win32(0))
    }
}
