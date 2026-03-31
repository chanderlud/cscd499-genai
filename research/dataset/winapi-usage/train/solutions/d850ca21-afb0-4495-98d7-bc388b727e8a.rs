#![allow(unused_imports)]
use windows::core::{Error, Result};
use windows::Win32::System::Kernel::{RtlFirstEntrySList, SLIST_ENTRY, SLIST_HEADER};

fn call_rtl_first_entry_s_list() -> Result<*mut SLIST_ENTRY> {
    // SAFETY: Zero-initializing SLIST_HEADER is valid for passing to RtlFirstEntrySList.
    let header: SLIST_HEADER = unsafe { std::mem::zeroed() };

    // SAFETY: RtlFirstEntrySList is unsafe and requires a valid pointer to an SLIST_HEADER.
    // We pass a reference to our initialized header.
    let result = unsafe { RtlFirstEntrySList(&header) };

    Ok(result)
}
