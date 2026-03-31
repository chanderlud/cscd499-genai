#[allow(unused_imports)]
use windows::core::{Error, Result};
use windows::Win32::System::Kernel::{RtlInterlockedFlushSList, SLIST_ENTRY, SLIST_HEADER};

fn call_rtl_interlocked_flush_s_list() -> Result<*mut SLIST_ENTRY> {
    // SAFETY: SLIST_HEADER is a plain data union; zero-initialization is valid.
    let mut header: SLIST_HEADER = unsafe { std::mem::zeroed() };

    // SAFETY: We pass a valid mutable pointer to an initialized SLIST_HEADER.
    // RtlInterlockedFlushSList is safe to call on a valid header.
    let result = unsafe { RtlInterlockedFlushSList(&mut header) };

    Ok(result)
}
