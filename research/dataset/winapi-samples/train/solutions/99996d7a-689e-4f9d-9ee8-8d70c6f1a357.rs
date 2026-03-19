use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Threading::{CreateEventW, WaitForSingleObject, INFINITE};

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

fn create_event_manually() -> Result<HANDLE> {
    let event_name = wide_null(OsStr::new("MyEvent"));

    // SAFETY: We're calling a Windows API function with valid parameters
    // CreateEventW returns Result<HANDLE> directly
    unsafe {
        CreateEventW(
            None,                        // Default security
            true,                        // Manual reset
            false,                       // Initial state: non-signaled
            PCWSTR(event_name.as_ptr()), // Event name
        )
    }
}

fn wait_for_event(handle: HANDLE) -> Result<()> {
    // SAFETY: We're calling a Windows API function with a valid handle
    let wait_result = unsafe { WaitForSingleObject(handle, INFINITE) };

    // WaitForSingleObject returns WAIT_EVENT, not a Result
    // WAIT_FAILED is 0xFFFFFFFF
    const WAIT_FAILED_VALUE: u32 = 0xFFFFFFFF;

    if wait_result.0 == WAIT_FAILED_VALUE {
        // Capture GetLastError() as a windows::core::Error
        Err(Error::from_thread())
    } else {
        Ok(())
    }
}

fn close_event_handle(handle: HANDLE) -> Result<()> {
    // SAFETY: We're calling a Windows API function with a valid handle
    // CloseHandle returns Result<()> directly
    unsafe { CloseHandle(handle) }
}

// Example function that demonstrates proper error handling with HRESULT
fn example_hresult_conversion() -> Result<()> {
    // Simulate getting a raw Win32 error code
    let raw_error_code: u32 = 5; // ERROR_ACCESS_DENIED

    // Convert raw u32 to HRESULT
    let hresult = HRESULT::from_win32(raw_error_code);

    // Convert HRESULT to Error for use with ? operator
    let _error = Error::from_hresult(hresult);

    // This would normally be used with ? operator:
    // some_function_that_returns_hresult()?;

    Ok(())
}

// Thread-safe example using strategy #2: Pass raw integer value across thread boundary
fn thread_safe_example() -> Result<()> {
    use std::thread;

    // Create event on main thread
    let event_handle = create_event_manually()?;

    // Extract raw handle value as usize for thread safety
    // usize is Send, while *mut c_void is not
    let raw_handle = event_handle.0 as usize;

    // Spawn thread with raw handle value
    let thread_handle = thread::spawn(move || {
        // Reconstruct typed HANDLE inside the thread
        let handle = HANDLE(raw_handle as *mut std::ffi::c_void);

        // Use the handle
        let result = wait_for_event(handle);

        // Close handle inside the thread before returning
        let _ = close_event_handle(handle);

        result
    });

    // Wait for thread to complete and get result
    thread_handle
        .join()
        .map_err(|_| Error::from_hresult(HRESULT::from_win32(1)))??;

    Ok(())
}

// Add a main function to use the functions and eliminate dead code warnings
fn main() -> Result<()> {
    let handle = create_event_manually()?;
    wait_for_event(handle)?;
    close_event_handle(handle)?;
    example_hresult_conversion()?;
    thread_safe_example()?;
    Ok(())
}
