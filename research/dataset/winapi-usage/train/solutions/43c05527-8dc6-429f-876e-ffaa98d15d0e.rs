use windows::core::Error;
use windows::Win32::Foundation::{CompareObjectHandles, HANDLE, WIN32_ERROR};

fn call_compare_object_handles() -> WIN32_ERROR {
    let h1 = HANDLE(std::ptr::null_mut());
    let h2 = HANDLE(std::ptr::null_mut());

    // SAFETY: CompareObjectHandles is safe to call with any HANDLE values.
    let success = unsafe { CompareObjectHandles(h1, h2) };

    if success.as_bool() {
        WIN32_ERROR(0)
    } else {
        let err = Error::from_thread();
        WIN32_ERROR(err.code().0 as u32)
    }
}
