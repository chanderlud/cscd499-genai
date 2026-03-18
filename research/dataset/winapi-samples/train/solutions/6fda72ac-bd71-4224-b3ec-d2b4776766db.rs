use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::sync::mpsc;
use std::thread;
use windows::core::{Error, Result, HRESULT, PCWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_GENERIC_READ, FILE_SHARE_READ, OPEN_EXISTING,
};
use windows::Win32::System::Threading::{CreateEventW, WaitForSingleObject, INFINITE};

fn wide_null(s: &OsStr) -> Vec<u16> {
    s.encode_wide().chain(once(0)).collect()
}

fn open_file(path: &OsStr) -> Result<HANDLE> {
    let wide_path = wide_null(path);
    let handle = unsafe {
        CreateFileW(
            PCWSTR(wide_path.as_ptr()),
            FILE_GENERIC_READ.0,
            FILE_SHARE_READ,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )?
    };
    if handle.is_invalid() {
        Err(Error::from_thread())
    } else {
        Ok(handle)
    }
}

fn create_event() -> Result<HANDLE> {
    let handle = unsafe { CreateEventW(None, true, false, PCWSTR::null())? };
    Ok(handle)
}

fn wait_for_event(event: HANDLE) -> Result<()> {
    let result = unsafe { WaitForSingleObject(event, INFINITE) };
    if result.0 == 0 {
        Ok(())
    } else {
        Err(Error::from_hresult(HRESULT::from_win32(result.0)))
    }
}

fn close_handle(handle: HANDLE) -> Result<()> {
    unsafe { CloseHandle(handle) }
}

fn main() -> Result<()> {
    let (tx, rx) = mpsc::channel();

    // Create event in main thread, pass raw value to worker
    let event_handle = create_event()?;
    let event_raw = event_handle.0 as isize;

    let worker = thread::spawn(move || {
        // Reconstruct HANDLE from raw value inside worker thread
        let event = HANDLE(event_raw as _);

        // Open file inside worker thread (handle stays local)
        let file_path = OsStr::new("test.txt");
        match open_file(file_path) {
            Ok(file_handle) => {
                // Simulate work
                thread::sleep(std::time::Duration::from_millis(100));

                // Signal completion
                let _ = tx.send(Ok(()));

                // Close file handle before returning
                let _ = close_handle(file_handle);
            }
            Err(e) => {
                let _ = tx.send(Err(e));
            }
        }

        // Signal event
        unsafe {
            windows::Win32::System::Threading::SetEvent(event).ok();
        }

        // Event handle will be closed when it goes out of scope
    });

    // Wait for worker to signal completion
    wait_for_event(event_handle)?;

    // Close event handle in main thread
    close_handle(event_handle)?;

    // Get result from worker
    let result = rx
        .recv()
        .map_err(|_| Error::from_hresult(HRESULT::from_win32(1)))?;

    // Wait for worker thread to finish
    worker
        .join()
        .map_err(|_| Error::from_hresult(HRESULT::from_win32(1)))?;

    result
}
