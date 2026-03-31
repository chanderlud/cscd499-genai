use windows::core::{Error, HRESULT};
use windows::Win32::System::Diagnostics::Debug::AddVectoredExceptionHandler;

fn call_add_vectored_exception_handler() -> HRESULT {
    // SAFETY: AddVectoredExceptionHandler is an unsafe Win32 API. We pass concrete values
    // and check the returned pointer for null to capture any thread-local error code.
    unsafe {
        let ptr = AddVectoredExceptionHandler(1, None);
        if ptr.is_null() {
            Error::from_thread().code()
        } else {
            HRESULT::default()
        }
    }
}
