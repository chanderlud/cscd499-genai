use windows::core::{Error, Result};
use windows::Win32::Foundation::{CloseHandle, HANDLE, WAIT_FAILED, WAIT_OBJECT_0, WAIT_TIMEOUT};
use windows::Win32::System::Threading::{CreateEventW, ResetEvent, SetEvent, WaitForSingleObject};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventStateReport {
    pub initial_sample_signaled: bool,
    pub after_set_sample_signaled: bool,
    pub after_reset_sample_signaled: bool,
}

struct OwnedHandle(HANDLE);

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        // SAFETY: CloseHandle is a Win32 API that requires unsafe
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

fn sample_signaled(handle: HANDLE) -> Result<bool> {
    // SAFETY: WaitForSingleObject is a Win32 API that requires unsafe
    unsafe {
        let wait_result = WaitForSingleObject(handle, 0);

        if wait_result == WAIT_OBJECT_0 {
            Ok(true)
        } else if wait_result == WAIT_TIMEOUT {
            Ok(false)
        } else {
            // WAIT_FAILED or other error - capture GetLastError()
            Err(Error::from_thread())
        }
    }
}

pub fn probe_manual_reset_event(initial_state: bool) -> Result<EventStateReport> {
    // SAFETY: CreateEventW is a Win32 API that requires unsafe
    unsafe {
        let event = OwnedHandle(CreateEventW(None, true, initial_state, None)?);

        let initial_sample_signaled = sample_signaled(event.0)?;

        SetEvent(event.0)?;
        let after_set_sample_signaled = sample_signaled(event.0)?;

        ResetEvent(event.0)?;
        let after_reset_sample_signaled = sample_signaled(event.0)?;

        Ok(EventStateReport {
            initial_sample_signaled,
            after_set_sample_signaled,
            after_reset_sample_signaled,
        })
    }
}
