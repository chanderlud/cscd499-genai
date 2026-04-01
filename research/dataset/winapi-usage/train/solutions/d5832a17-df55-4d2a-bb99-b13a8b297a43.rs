use windows::Win32::Foundation::{GetLastError, WIN32_ERROR};
use windows::Win32::System::Kernel::{RtlInterlockedPushEntrySList, SLIST_ENTRY, SLIST_HEADER};

fn call_rtl_interlocked_push_entry_s_list() -> WIN32_ERROR {
    let mut list_head: SLIST_HEADER = unsafe { std::mem::zeroed() };
    let mut list_entry: SLIST_ENTRY = unsafe { std::mem::zeroed() };

    let result = unsafe { RtlInterlockedPushEntrySList(&mut list_head, &mut list_entry) };

    if result.is_null() {
        let error_code = unsafe { GetLastError().0 };
        WIN32_ERROR(error_code)
    } else {
        WIN32_ERROR(0)
    }
}
