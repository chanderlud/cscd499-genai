use windows::core::Error;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Diagnostics::Debug::AddVectoredExceptionHandler;

fn call_add_vectored_exception_handler() -> WIN32_ERROR {
    let handler = unsafe { AddVectoredExceptionHandler(0, None) };
    if handler.is_null() {
        WIN32_ERROR(Error::from_thread().code().0 as u32)
    } else {
        WIN32_ERROR(0)
    }
}
