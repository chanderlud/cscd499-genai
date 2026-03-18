use windows::core::{Error, Result};
use windows::Win32::Foundation::{CloseHandle, E_INVALIDARG, HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT};
use windows::Win32::System::Threading::{
    CreateWaitableTimerW, SetWaitableTimer, WaitForSingleObjectEx, PTIMERAPCROUTINE,
};

unsafe extern "system" fn apc_routine(
    _context: *const std::ffi::c_void,
    _timer_low: u32,
    _timer_high: u32,
) {
    let fired_ptr = _context as *mut bool;
    *fired_ptr = true;
}

pub fn apc_timer_fires(due_ms: i64, timeout_ms: u32) -> Result<bool> {
    let mut fired = false;
    let mut timer_handle = HANDLE::default();

    unsafe {
        timer_handle = CreateWaitableTimerW(None, false, None)?;
    }

    let due_time = due_ms.checked_mul(-10000).ok_or_else(|| {
        unsafe { CloseHandle(timer_handle).ok() };
        Error::from_hresult(E_INVALIDARG)
    })?;

    unsafe {
        let result = SetWaitableTimer(
            timer_handle,
            &due_time,
            0,
            Some(apc_routine),
            Some(&mut fired as *mut bool as *const std::ffi::c_void),
            false,
        );
        if let Err(err) = result {
            CloseHandle(timer_handle).ok();
            return Err(err);
        }
    }

    let wait_result = unsafe { WaitForSingleObjectEx(timer_handle, timeout_ms, true) };

    unsafe {
        CloseHandle(timer_handle).ok();
    }

    match wait_result {
        WAIT_OBJECT_0 => Ok(fired),
        WAIT_TIMEOUT => Ok(fired),
        _ => Err(Error::from_thread()),
    }
}
