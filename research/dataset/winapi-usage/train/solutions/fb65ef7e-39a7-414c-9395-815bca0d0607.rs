use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Kernel::{RtlInterlockedFlushSList, SLIST_HEADER};

fn call_rtl_interlocked_flush_s_list() -> WIN32_ERROR {
    // SAFETY: RtlInterlockedFlushSList is an unsafe FFI function. We pass a null pointer
    // as a concrete parameter value per task requirements.
    unsafe {
        RtlInterlockedFlushSList(std::ptr::null_mut::<SLIST_HEADER>());
    }
    WIN32_ERROR(0)
}
