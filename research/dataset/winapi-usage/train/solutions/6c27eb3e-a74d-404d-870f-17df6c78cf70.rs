use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::UI::HiDpi::{
    AreDpiAwarenessContextsEqual, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};

fn call_are_dpi_awareness_contexts_equal() -> WIN32_ERROR {
    // SAFETY: Passing valid DPI_AWARENESS_CONTEXT constants to the API.
    let _ = unsafe {
        AreDpiAwarenessContextsEqual(
            DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
            DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
        )
    };
    WIN32_ERROR(0)
}
