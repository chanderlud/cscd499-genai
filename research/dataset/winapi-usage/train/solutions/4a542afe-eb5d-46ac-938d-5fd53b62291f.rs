use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Diagnostics::Debug::{
    AddVectoredContinueHandler, PVECTORED_EXCEPTION_HANDLER,
};

fn call_add_vectored_continue_handler() -> WIN32_ERROR {
    unsafe {
        let handler: PVECTORED_EXCEPTION_HANDLER = None;
        let ptr = AddVectoredContinueHandler(0, handler);
        if ptr.is_null() {
            let err = Error::from_thread();
            WIN32_ERROR(err.code().0 as u32)
        } else {
            WIN32_ERROR(0)
        }
    }
}
