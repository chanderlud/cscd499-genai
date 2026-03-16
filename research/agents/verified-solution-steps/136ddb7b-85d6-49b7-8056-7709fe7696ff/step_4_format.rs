use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, Ordering};
use windows::core::{Error, Result};
use windows::Win32::Foundation::{CloseHandle, E_INVALIDARG, HANDLE, WAIT_FAILED};
use windows::Win32::System::Threading::{CreateWaitableTimerW, SetWaitableTimer, SleepEx};

unsafe extern "system" fn timer_apc_completion_routine(
    lparg_to_completion_routine: *const c_void,
    _dw_timer_low_value: u32,
    _dw_timer_high_value: u32,
) {
    // SAFETY: The pointer is valid for the duration of the APC call
    let fired = &*(lparg_to_completion_routine as *const AtomicBool);
    fired.store(true, Ordering::Release);
}

pub fn apc_timer_fires(due_ms: i64, timeout_ms: u32) -> Result<bool> {
    // Create waitable timer
    let timer_handle = unsafe { CreateWaitableTimerW(None, false, None)? };

    // Ensure timer handle is closed on exit
    struct TimerGuard(HANDLE);
    impl Drop for TimerGuard {
        fn drop(&mut self) {
            unsafe {
                let _ = CloseHandle(self.0);
            }
        }
    }
    let _guard = TimerGuard(timer_handle);

    // Track whether APC callback fired
    let fired = AtomicBool::new(false);
    let fired_ptr = &fired as *const AtomicBool as *const c_void;

    // Convert milliseconds to 100-nanosecond intervals (negative for relative time)
    let due_time = due_ms.checked_mul(-10000).ok_or_else(|| {
        Error::from_hresult(E_INVALIDARG) // E_INVALIDARG
    })?;

    // Set timer with APC completion routine
    unsafe {
        SetWaitableTimer(
            timer_handle,
            &due_time,
            0, // No period (one-shot)
            Some(timer_apc_completion_routine),
            Some(fired_ptr),
            false,
        )?;
    }

    // Enter alertable wait
    let wait_result = unsafe { SleepEx(timeout_ms, true) };
    if wait_result == WAIT_FAILED.0 {
        return Err(Error::from_thread());
    }

    Ok(fired.load(Ordering::Acquire))
}