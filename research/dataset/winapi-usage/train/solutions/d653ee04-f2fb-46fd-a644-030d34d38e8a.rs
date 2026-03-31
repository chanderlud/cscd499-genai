use windows::core::HRESULT;
use windows::core::{Error, Result};
use windows::Win32::System::Kernel::RtlFirstEntrySList;

fn call_rtl_first_entry_s_list() -> HRESULT {
    // SAFETY: Passing a null pointer is safe for this demonstration;
    // the API simply returns a pointer without dereferencing it in a way that causes immediate UB.
    let _ = unsafe { RtlFirstEntrySList(std::ptr::null()) };
    HRESULT(0)
}
