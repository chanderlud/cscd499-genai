use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, ERROR_ACCESS_DENIED, HANDLE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_FLAGS_AND_ATTRIBUTES, FILE_GENERIC_READ, FILE_SHARE_READ, OPEN_EXISTING,
};
use windows::Win32::System::Threading::{CreateThread, THREAD_CREATION_FLAGS};

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

// Helper to open a file handle safely within a thread
fn open_file_in_thread(path: &OsStr) -> Result<HANDLE> {
    let wide_path = wide_null(path);
    // SAFETY: CreateFileW is called with valid parameters and the result is checked
    let handle = unsafe {
        CreateFileW(
            PCWSTR(wide_path.as_ptr()),
            FILE_GENERIC_READ.0,
            FILE_SHARE_READ,
            None,
            OPEN_EXISTING,
            FILE_FLAGS_AND_ATTRIBUTES(0),
            None,
        )
    }?; // Use ? to extract HANDLE from Result

    Ok(handle)
}

// Thread worker function - handle is created and closed entirely within the thread
fn file_reader_thread(path: String) -> Result<()> {
    let os_path = OsStr::new(&path);
    let file_handle = open_file_in_thread(os_path)?;

    // SAFETY: We have a valid handle and will close it before returning
    unsafe {
        // In a real implementation, we would read from the file here
        // For this example, we just close the handle immediately
        let _ = CloseHandle(file_handle);
    }

    Ok(())
}

// Example of converting a WIN32_ERROR to HRESULT
fn check_access_denied() -> Result<()> {
    // Simulate a Win32 error code
    let error_code = ERROR_ACCESS_DENIED.0;
    let hresult = HRESULT::from_win32(error_code);
    Err(Error::from_hresult(hresult))
}

// Main function demonstrating thread creation with handle isolation
fn main() -> Result<()> {
    let file_path = String::from("C:\\Windows\\System32\\notepad.exe");

    // Strategy 1: Create handle inside thread closure
    // Pass only the Send-safe String to the thread
    let thread_handle = unsafe {
        CreateThread(
            None,
            0,
            Some(thread_proc),
            // Pass the path as a raw pointer to Box<String>
            Some(Box::into_raw(Box::new(file_path)) as *const std::ffi::c_void), // Fixed: wrapped in Some()
            THREAD_CREATION_FLAGS(0),
            None,
        )
    }?;

    // SAFETY: We have a valid thread handle and will wait for it
    unsafe {
        windows::Win32::System::Threading::WaitForSingleObject(
            thread_handle,
            windows::Win32::System::Threading::INFINITE,
        );
        let _ = CloseHandle(thread_handle);
    }

    // Demonstrate HRESULT conversion
    if let Err(e) = check_access_denied() {
        println!("Expected access denied error: {:?}", e);
    }

    Ok(())
}

// Thread procedure that reconstructs the String from raw pointer
// SAFETY: This function is only called once with the pointer from Box::into_raw
unsafe extern "system" fn thread_proc(param: *mut std::ffi::c_void) -> u32 {
    // Reconstruct the Box<String> from the raw pointer
    let path_box = Box::from_raw(param as *mut String);
    let path = *path_box;

    match file_reader_thread(path) {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("Thread error: {}", e);
            1
        }
    }
}
