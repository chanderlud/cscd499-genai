use windows::core::{Error, Result, HRESULT};
use windows::Win32::Foundation::S_OK;
use windows::Win32::UI::HiDpi::{AreDpiAwarenessContextsEqual, DPI_AWARENESS_CONTEXT};

fn call_are_dpi_awareness_contexts_equal() -> HRESULT {
    unsafe {
        let _ = AreDpiAwarenessContextsEqual(
            DPI_AWARENESS_CONTEXT(std::ptr::null_mut()),
            DPI_AWARENESS_CONTEXT(std::ptr::null_mut()),
        );
    }
    S_OK
}
