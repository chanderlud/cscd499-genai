use windows::core::{Error, Result};
use windows::Win32::System::Threading::{AcquireSRWLockExclusive, SRWLOCK};

fn call_acquire_srw_lock_exclusive() -> Result<()> {
    let mut lock = SRWLOCK::default();
    // SAFETY: `lock` is zero-initialized, which is equivalent to SRWLOCK_INIT and satisfies the API's precondition.
    unsafe { AcquireSRWLockExclusive(&mut lock) };
    Ok(())
}
