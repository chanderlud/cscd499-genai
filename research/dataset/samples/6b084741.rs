use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use windows::core::{Error, Result, PWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};

fn get_process_name_from_pid(pid: u32) -> Result<String> {
    unsafe {
        // Open process with query information access
        let handle: HANDLE = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid)?;

        // Query process image name
        let mut size: u32 = 260; // MAX_PATH
        let mut buffer: Vec<u16> = vec![0; size as usize];

        let result = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut size,
        );

        // Close the handle regardless of success/failure
        let _ = CloseHandle(handle);

        result?;

        if size > 0 {
            // Convert to OsString and then to String
            let os_string = OsString::from_wide(&buffer[..size as usize]);
            let path_str = os_string.to_string_lossy().to_string();

            // Extract just the filename
            if let Some(filename) = path_str.split('\\').next_back() {
                return Ok(filename.to_string());
            }
        }

        // If we reach here, something went wrong with the path extraction
        // Use from_thread() to capture GetLastError() as an Error
        Err(Error::from_thread())
    }
}

fn main() -> Result<()> {
    // Example: Get process name for current process
    let current_pid = std::process::id();
    println!("Current process PID: {}", current_pid);

    match get_process_name_from_pid(current_pid) {
        Ok(name) => println!("Process name: {}", name),
        Err(e) => println!("Failed to get process name: {}", e),
    }

    Ok(())
}
