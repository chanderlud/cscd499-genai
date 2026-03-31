use windows::core::{Error, Result, HRESULT};
use windows::Win32::Graphics::Gdi::AddFontMemResourceEx;

fn call_add_font_mem_resource_ex() -> HRESULT {
    // SAFETY: Calling AddFontMemResourceEx with dummy parameters.
    // We check the returned HANDLE for validity and capture GetLastError on failure.
    let handle = unsafe {
        AddFontMemResourceEx(
            std::ptr::null::<core::ffi::c_void>(),
            0,
            None,
            std::ptr::null::<u32>(),
        )
    };
    if handle.0.is_null() {
        Error::from_thread().code()
    } else {
        HRESULT(0)
    }
}
