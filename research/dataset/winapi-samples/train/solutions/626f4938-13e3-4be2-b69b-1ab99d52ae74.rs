use windows::core::{Error, Result};
use windows::Win32::Foundation::{CloseHandle, HANDLE, WAIT_FAILED, WAIT_OBJECT_0, WAIT_TIMEOUT};
use windows::Win32::System::Threading::{CreateEventW, SetEvent, WaitForSingleObject};

#[derive(Debug)]
enum WaitResult {
    Signaled,
    Timeout,
}

fn create_set_and_wait_event(timeout_ms: u32) -> Result<WaitResult> {
    let event = unsafe { CreateEventW(Some(std::ptr::null()), true, false, None)? };

    struct EventHandle(HANDLE);
    impl Drop for EventHandle {
        fn drop(&mut self) {
            unsafe {
                let _ = CloseHandle(self.0);
            };
        }
    }
    let event_guard = EventHandle(event);

    unsafe {
        SetEvent(event)?;
    }

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
