use windows::core::{Error, Result, HRESULT};
use windows::Win32::System::Threading::{AcquireSRWLockExclusive, SRWLOCK};

fn call_acquire_srw_lock_exclusive() -> HRESULT {
    let mut lock = SRWLOCK {
        Ptr: std::ptr::null_mut(),
    };
    unsafe {
        AcquireSRWLockExclusive(&mut lock);
    }
    HRESULT::from_win32(0)
}
