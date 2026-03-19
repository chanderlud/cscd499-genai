use windows::core::{Error, Result};
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};

fn find_explorer_process() -> Result<bool> {
    // SAFETY: CreateToolhelp32Snapshot returns a handle that must be closed
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }?;

    if snapshot.is_invalid() {
        return Err(Error::from_thread());
    }

    let mut found = false;
    let mut process_entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };

    // SAFETY: Process32FirstW requires a valid snapshot handle and properly initialized PROCESSENTRY32W
    if unsafe { Process32FirstW(snapshot, &mut process_entry) }.is_ok() {
        loop {
            // Extract process name from null-terminated UTF-16 string
            let name_len = process_entry
                .szExeFile
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(0);
            let name = String::from_utf16_lossy(&process_entry.szExeFile[..name_len]);

            if name.eq_ignore_ascii_case("explorer.exe") {
                found = true;
                break;
            }

            // SAFETY: Process32NextW requires a valid snapshot handle
            if unsafe { Process32NextW(snapshot, &mut process_entry) }.is_err() {
                break;
            }
        }
    }

    // SAFETY: CloseHandle requires a valid handle
    unsafe {
        let _ = CloseHandle(snapshot);
    };

    Ok(found)
}

fn main() -> Result<()> {
    match find_explorer_process() {
        Ok(true) => println!("Found explorer.exe process"),
        Ok(false) => println!("explorer.exe process not found"),
        Err(e) => eprintln!("Error searching for processes: {}", e),
    }
    Ok(())
}
