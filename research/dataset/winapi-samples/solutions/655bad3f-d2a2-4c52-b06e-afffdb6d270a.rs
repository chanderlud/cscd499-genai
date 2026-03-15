use windows::core::{Error, Result};
use windows::Win32::Foundation::{CloseHandle, ERROR_NO_MORE_FILES, INVALID_HANDLE_VALUE};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};

fn main() -> Result<()> {
    // Create a snapshot of all processes
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)? };

    if snapshot == INVALID_HANDLE_VALUE {
        return Err(Error::from_thread());
    }

    // Initialize the PROCESSENTRY32W structure with its size
    let mut entry = PROCESSENTRY32W {
        dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };

    // Get the first process
    if unsafe { Process32FirstW(snapshot, &mut entry) }.is_ok() {
        println!("First process found:");
        print_process_entry(&entry);
    } else {
        eprintln!("Failed to get first process");
        unsafe { CloseHandle(snapshot)? };
        return Err(Error::from_thread());
    }

    // Iterate through remaining processes
    loop {
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        match unsafe { Process32NextW(snapshot, &mut entry) } {
            Ok(_) => {
                println!("\nNext process found:");
                print_process_entry(&entry);
            }
            Err(e) if e.code() == ERROR_NO_MORE_FILES.into() => break,
            Err(e) => {
                eprintln!("Error getting next process: {}", e);
                unsafe { CloseHandle(snapshot)? };
                return Err(e);
            }
        }
    }

    // Clean up the snapshot handle
    unsafe { CloseHandle(snapshot)? };

    Ok(())
}

fn print_process_entry(entry: &PROCESSENTRY32W) {
    let pid = entry.th32ProcessID;
    println!("  Process ID: {}", pid);

    let name = if let Some(len) = entry.szExeFile.iter().position(|&c| c == 0) {
        String::from_utf16_lossy(&entry.szExeFile[..len])
    } else {
        String::from_utf16_lossy(&entry.szExeFile)
    };
    println!("  Process Name: {}", name);

    let parent_pid = entry.th32ParentProcessID;
    println!("  Parent Process ID: {}", parent_pid);
}
