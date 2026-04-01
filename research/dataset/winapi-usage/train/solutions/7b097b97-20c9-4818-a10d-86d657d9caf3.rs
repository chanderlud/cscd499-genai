use windows::core::{Error, Result};
use windows::Win32::System::Kernel::{RtlInterlockedPushEntrySList, SLIST_ENTRY, SLIST_HEADER};

fn call_rtl_interlocked_push_entry_s_list() -> Result<*mut SLIST_ENTRY> {
    // Create concrete parameter values with zeroed memory
    // SAFETY: We're creating valid SLIST_HEADER and SLIST_ENTRY structures
    let mut list_head = unsafe { std::mem::zeroed::<SLIST_HEADER>() };
    let mut list_entry = unsafe { std::mem::zeroed::<SLIST_ENTRY>() };

    // Call the API
    // SAFETY: We're passing valid pointers to SLIST_HEADER and SLIST_ENTRY
    let result = unsafe { RtlInterlockedPushEntrySList(&mut list_head, &mut list_entry) };

    // Check for NULL (failure case)
    if result.is_null() {
        Err(Error::from_thread())
    } else {
        Ok(result)
    }
}
