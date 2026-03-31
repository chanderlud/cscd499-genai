use windows::core::{Error, Result, HRESULT};
use windows::Win32::System::Kernel::RtlInitializeSListHead;

fn call_rtl_initialize_s_list_head() -> HRESULT {
    unsafe {
        RtlInitializeSListHead();
    }
    HRESULT(0)
}
