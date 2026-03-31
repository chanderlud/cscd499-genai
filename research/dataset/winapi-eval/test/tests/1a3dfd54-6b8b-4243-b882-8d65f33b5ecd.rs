#[cfg(test)]
mod tests {
  use super::signal_event;
  use windows::core::Result;
  use windows::Win32::Foundation::{CloseHandle, WAIT_OBJECT_0};
  use windows::Win32::System::Threading::{
    CreateEventW, WaitForSingleObject,
  };

  #[test]
  fn signal_event_sets_event_to_signaled_state() -> Result<()> {
    let event = unsafe { CreateEventW(None, true, false, None)? };

    signal_event(event)?;

    let wait_result = unsafe { WaitForSingleObject(event, 0) };
    assert_eq!(wait_result, WAIT_OBJECT_0);

    unsafe { CloseHandle(event)? };
    Ok(())
  }

  #[test]
  fn signal_event_on_already_signaled_manual_reset_event_still_succeeds() -> Result<()> {
    let event = unsafe { CreateEventW(None, true, true, None)? };

    signal_event(event)?;

    let wait_result = unsafe { WaitForSingleObject(event, 0) };
    assert_eq!(wait_result, WAIT_OBJECT_0);

    unsafe { CloseHandle(event)? };
    Ok(())
  }
}
