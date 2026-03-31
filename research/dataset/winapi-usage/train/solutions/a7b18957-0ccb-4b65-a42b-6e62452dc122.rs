use windows::core::{Error, Result};
use windows::Win32::System::Diagnostics::Debug::{
    AddVectoredContinueHandler, EXCEPTION_POINTERS, PVECTORED_EXCEPTION_HANDLER,
};

extern "system" fn vectored_continue_handler(_exception_info: *mut EXCEPTION_POINTERS) -> i32 {
    0
}

fn call_add_vectored_continue_handler() -> Result<*mut core::ffi::c_void> {
    let handler: PVECTORED_EXCEPTION_HANDLER = Some(vectored_continue_handler);
    // SAFETY: The handler function is valid and matches the expected signature.
    let result = unsafe { AddVectoredContinueHandler(0, handler) };
    if result.is_null() {
        return Err(Error::from_thread());
    }
    Ok(result)
}
