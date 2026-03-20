use windows::core::{Error, Result, PCWSTR};
use windows::Win32::Foundation::{
    CloseHandle, FILETIME, HANDLE, WAIT_FAILED, WAIT_OBJECT_0, WAIT_TIMEOUT,
};
use windows::Win32::System::Threading::{
    CloseThreadpoolTimer, CreateEventW, CreateThreadpoolTimer, SetEvent, SetThreadpoolTimer,
    WaitForSingleObject, WaitForThreadpoolTimerCallbacks, PTP_TIMER,
};

unsafe extern "system" fn timer_callback(
    _instance: windows::Win32::System::Threading::PTP_CALLBACK_INSTANCE,
    context: *mut std::ffi::c_void,
    _timer: PTP_TIMER,
) {
    // SAFETY: context is the event handle passed at timer creation time
    let event = HANDLE(context);
    let _ = unsafe { SetEvent(event) };
}

pub fn tp_timer_signal(due_ms: u32, timeout_ms: u32) -> Result<bool> {
    // Create a manual-reset event (initially non-signaled)
    let event = unsafe {
        CreateEventW(
            None,
            true,           // manual reset
            false,          // initially non-signaled
            PCWSTR::null(), // unnamed
        )?
    };

    // Create the timer, making sure we don't leak the event on failure
    let timer = unsafe {
        match CreateThreadpoolTimer(
            Some(timer_callback),
            Some(event.0), // Removed unnecessary cast
            None,
        ) {
            Ok(timer) => timer,
            Err(e) => {
                let _ = CloseHandle(event);
                return Err(e);
            }
        }
    };

    // Relative due time:
    // negative value = delay relative to "now", in 100ns units
    let due_100ns = -(i64::from(due_ms) * 10_000);

    let due_time = FILETIME {
        dwLowDateTime: due_100ns as u32,
        dwHighDateTime: (due_100ns >> 32) as u32,
    };

    unsafe {
        SetThreadpoolTimer(
            timer,
            Some(&due_time as *const FILETIME),
            0,       // one-shot
            Some(0), // no coalescing window
        );
    }

    let wait_result = unsafe { WaitForSingleObject(event, timeout_ms) };

    unsafe {
        // Cancel further callbacks and drain any queued/running callback
        SetThreadpoolTimer(timer, None, 0, None);
        WaitForThreadpoolTimerCallbacks(timer, true);
        CloseThreadpoolTimer(timer);
        CloseHandle(event)?;
    }

    match wait_result {
        WAIT_OBJECT_0 => Ok(true),
        WAIT_TIMEOUT => Ok(false),
        WAIT_FAILED => Err(Error::from_thread()),
        _ => Err(Error::from_thread()),
    }
}
