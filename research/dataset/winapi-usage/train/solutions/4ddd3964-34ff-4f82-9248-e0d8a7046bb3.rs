use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Threading::{AcquireSRWLockExclusive, SRWLOCK_INIT};

fn call_acquire_srw_lock_exclusive() -> WIN32_ERROR {
    let mut lock = SRWLOCK_INIT;
    unsafe {
        AcquireSRWLockExclusive(&mut lock);
    }
    WIN32_ERROR(0)
}
