use windows::{
    core::{Error, Result, HRESULT},
    Win32::{
        Foundation::{CloseHandle, HANDLE},
        System::Threading::{CreateEventW, ResetEvent, SetEvent, WaitForSingleObject},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventAction {
    Set,
    Reset,
    Check,
}

struct OwnedEvent(HANDLE);

impl Drop for OwnedEvent {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

fn is_signaled(handle: HANDLE) -> Result<bool> {
    let wait_result = unsafe { WaitForSingleObject(handle, 0) };

    match wait_result.0 {
        0 => Ok(true),    // WAIT_OBJECT_0
        258 => Ok(false), // WAIT_TIMEOUT
        _ => Err(Error::from_hresult(HRESULT::from_win32(wait_result.0))),
    }
}

pub fn run_manual_reset_event_script(
    initial_state: bool,
    actions: &[EventAction],
) -> Result<Vec<bool>> {
    let event = unsafe { OwnedEvent(CreateEventW(None, true, initial_state, None)?) };

    let mut observations = Vec::with_capacity(actions.len());

    for action in actions {
        match action {
            EventAction::Set => unsafe {
                SetEvent(event.0)?;
            },
            EventAction::Reset => unsafe {
                ResetEvent(event.0)?;
            },
            EventAction::Check => {}
        }

        observations.push(is_signaled(event.0)?);
    }

    Ok(observations)
}
