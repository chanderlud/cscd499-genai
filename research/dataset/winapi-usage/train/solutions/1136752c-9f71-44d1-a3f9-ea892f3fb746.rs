use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::System::Kernel::{RtlFirstEntrySList, SLIST_HEADER};

fn call_rtl_first_entry_s_list() -> WIN32_ERROR {
    unsafe {
        let header = std::mem::zeroed::<SLIST_HEADER>();
        RtlFirstEntrySList(&header);
    }
    WIN32_ERROR(0)
}
