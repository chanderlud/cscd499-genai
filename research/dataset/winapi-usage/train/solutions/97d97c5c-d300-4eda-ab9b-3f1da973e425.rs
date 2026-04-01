use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::WindowsProgramming::AddDelBackupEntryW;

unsafe fn call_add_del_backup_entry_w() -> WIN32_ERROR {
    let file_list = windows::core::w!("C:\\temp\\filelist.txt");
    let backup_dir = windows::core::w!("C:\\temp\\backup");
    let base_name = windows::core::w!("backup");
    let flags = 0u32;

    match AddDelBackupEntryW(file_list, backup_dir, base_name, flags) {
        Ok(_) => WIN32_ERROR(0),
        Err(e) => WIN32_ERROR(e.code().0 as u32),
    }
}
