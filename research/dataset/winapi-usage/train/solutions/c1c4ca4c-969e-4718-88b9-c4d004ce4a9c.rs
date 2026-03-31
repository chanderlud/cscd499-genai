use windows::core::{Error, Result};
use windows::Win32::System::Kernel::{RtlInitializeSListHead, SLIST_HEADER};

fn call_rtl_initialize_s_list_head() -> Result<SLIST_HEADER> {
    // SAFETY: RtlInitializeSListHead is a standard Win32 API that initializes an SLIST_HEADER.
    // It does not fail and requires no special preconditions.
    Ok(unsafe { RtlInitializeSListHead() })
}
