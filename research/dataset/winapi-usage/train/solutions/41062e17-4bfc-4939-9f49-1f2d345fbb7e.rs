#[allow(unused_imports)]
use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::S_OK;
use windows::Win32::System::Kernel::{RtlInterlockedFlushSList, SLIST_HEADER};

fn call_rtl_interlocked_flush_s_list() -> HRESULT {
    unsafe {
        RtlInterlockedFlushSList(std::ptr::null_mut::<SLIST_HEADER>());
        S_OK
    }
}
