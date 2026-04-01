use windows::core::{Error, HRESULT};
use windows::Win32::System::Kernel::{RtlInterlockedPushEntrySList, SLIST_ENTRY, SLIST_HEADER};

fn call_rtl_interlocked_push_entry_s_list() -> HRESULT {
    // Create concrete instances of SLIST_HEADER and SLIST_ENTRY
    let mut header = SLIST_HEADER {
        Anonymous: Default::default(),
    };
    let mut entry = SLIST_ENTRY {
        Next: Default::default(),
    };

    // Call the unsafe API with concrete parameter values
    let result = unsafe { RtlInterlockedPushEntrySList(&mut header, &mut entry) };

    // Check if the result pointer is null (indicates failure)
    if result.is_null() {
        // Capture GetLastError() as HRESULT
        Error::from_thread().code()
    } else {
        // Success - return S_OK
        HRESULT::from_win32(0)
    }
}
