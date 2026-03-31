use windows::core::{Error, Result};
use windows::Win32::System::Diagnostics::Debug::{AddVectoredExceptionHandler, EXCEPTION_POINTERS};

unsafe extern "system" fn vectored_handler(_info: *mut EXCEPTION_POINTERS) -> i32 {
    0
}

fn call_add_vectored_exception_handler() -> Result<*mut core::ffi::c_void> {
    let ptr = unsafe { AddVectoredExceptionHandler(1, Some(vectored_handler)) };
    if ptr.is_null() {
        Err(Error::from_thread())
    } else {
        Ok(ptr)
    }
}
