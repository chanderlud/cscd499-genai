use windows::{core::Result, Win32::System::Threading::*};

static COUNTER: std::sync::RwLock<i32> = std::sync::RwLock::new(0);

fn main() -> Result<()> {
    // SAFETY: CreateThreadpoolWork is a safe wrapper that returns a Result
    let work = unsafe { CreateThreadpoolWork(Some(callback), None, None)? };

    // SAFETY: SubmitThreadpoolWork is an unsafe FFI call that doesn't return a Result
    for _ in 0..10 {
        unsafe {
            SubmitThreadpoolWork(work);
        }
    }

    // SAFETY: WaitForThreadpoolWorkCallbacks is an unsafe FFI call that doesn't return a Result
    unsafe {
        WaitForThreadpoolWorkCallbacks(work, false);
    }

    let counter = COUNTER.read().unwrap();
    println!("counter: {}", *counter);
    Ok(())
}

unsafe extern "system" fn callback(
    _: PTP_CALLBACK_INSTANCE,
    _: *mut std::ffi::c_void,
    _: PTP_WORK,
) {
    let mut counter = COUNTER.write().unwrap();
    *counter += 1;
}
