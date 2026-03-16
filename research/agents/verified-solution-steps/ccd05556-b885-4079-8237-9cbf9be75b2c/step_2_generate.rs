use windows::core::{Result, Error, HRESULT, PCWSTR};
use windows::Win32::Foundation::{HANDLE, WIN32_ERROR, ERROR_SUCCESS, CloseHandle};
use windows::Win32::Storage::FileSystem::{CreateFileW, FILE_GENERIC_WRITE, FILE_SHARE_READ, OPEN_ALWAYS, FILE_ATTRIBUTE_NORMAL};
use windows::Win32::System::Threading::{CreateThread, THREAD_CREATE_RUN_IMMEDIATELY, THREAD_CREATION_FLAGS};
use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::ptr;

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

fn create_file(path: &OsStr) -> Result<HANDLE> {
    let wide_path = wide_null(path);
    let handle = unsafe {
        CreateFileW(
            PCWSTR(wide_path.as_ptr()),
            FILE_GENERIC_WRITE.0,
            FILE_SHARE_READ,
            ptr::null(),
            OPEN_ALWAYS,
            FILE_ATTRIBUTE_NORMAL,
            HANDLE::default(),
        )
    };
    
    if handle.is_invalid() {
        // Capture GetLastError() as a windows::core::Error
        return Err(Error::from_thread());
    }
    
    Ok(handle)
}

fn write_to_file_thread(handle_raw: isize) -> Result<()> {
    // Reconstruct the HANDLE from the raw integer value
    let handle = HANDLE(handle_raw);
    
    // Simulate some work with the handle
    // In a real scenario, you would perform file operations here
    
    // Close the handle when done
    unsafe {
        CloseHandle(handle)?;
    }
    
    Ok(())
}

fn main() -> Result<()> {
    let file_path = OsStr::new("test.txt");
    let file_handle = create_file(file_path)?;
    
    // Extract the raw handle value before spawning the thread
    let handle_raw = file_handle.0;
    
    // Spawn a thread that uses the raw handle value
    let thread_handle = unsafe {
        CreateThread(
            ptr::null(),
            0,
            Some(write_to_file_thread_wrapper),
            Some(&handle_raw as *const _ as *const _),
            THREAD_CREATE_RUN_IMMEDIATELY,
            ptr::null_mut(),
        )
    }?;
    
    // Wait for thread to complete
    unsafe {
        windows::Win32::System::Threading::WaitForSingleObject(thread_handle, windows::Win32::System::Threading::INFINITE);
        CloseHandle(thread_handle)?;
    }
    
    Ok(())
}

// Wrapper function for the thread that matches the required signature
unsafe extern "system" fn write_to_file_thread_wrapper(param: *mut std::ffi::c_void) -> u32 {
    let handle_raw = *(param as *const isize);
    
    match write_to_file_thread(handle_raw) {
        Ok(()) => ERROR_SUCCESS.0,
        Err(e) => {
            // Convert the error to a WIN32_ERROR code
            let hresult: HRESULT = e.into();
            // Extract the Win32 error code from the HRESULT
            let win32_code = (hresult.0 & 0xFFFF) as u32;
            win32_code
        }
    }
}