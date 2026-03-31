use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Kernel::RtlInitializeSListHead;

fn call_rtl_initialize_s_list_head() -> WIN32_ERROR {
    // SAFETY: RtlInitializeSListHead is a simple initializer that does not fail.
    let _header = unsafe { RtlInitializeSListHead() };
    WIN32_ERROR(0)
}
