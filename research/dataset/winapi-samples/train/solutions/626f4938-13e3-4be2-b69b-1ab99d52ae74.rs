use windows::core::{Error, Result};
use windows::Win32::Foundation::{CloseHandle, HANDLE, WAIT_FAILED, WAIT_OBJECT_0, WAIT_TIMEOUT};
use windows::Win32::System::Threading::{CreateEventW, SetEvent, WaitForSingleObject};

enum WaitResult {
    Signaled,
    Timeout,
}

fn create_set_and_wait_event(timeout_ms: u32) -> Result<WaitResult> {
    // Create manual-reset event in non-signaled state
    // SAFETY: Creating an event with valid parameters
    let event = unsafe {
        CreateEventW(
            Some(std::ptr::null()), // Default security attributes (wrapped in Some)
            true,                   // Manual-reset
            false,                  // Initial state: non-signaled
            None,                   // Unnamed event
        )?
    };

    // Ensure we clean up the event handle even if an error occurs
    struct EventHandle(HANDLE);
    impl Drop for EventHandle {
        fn drop(&mut self) {
            // SAFETY: Closing a valid event handle
            unsafe {
                let _ = CloseHandle(self.0);
            };
        }
    }
    let event_guard = EventHandle(event);

    // Set event to signaled state
    // SAFETY: Setting a valid event handle
    unsafe {
        SetEvent(event)?;
    }

    // Wait for the event with timeout
    // SAFETY: Waiting on a valid event handle
    let wait_result = unsafe { WaitForSingleObject(event_guard.0, timeout_ms) };

    match wait_result {
        WAIT_OBJECT_0 => Ok(WaitResult::Signaled),
        WAIT_TIMEOUT => Ok(WaitResult::Timeout),
        WAIT_FAILED => Err(Error::from_thread()),
        other => Err(Error::from_hresult(windows::core::HRESULT::from_win32(
            other.0,
        ))),
    }
}
