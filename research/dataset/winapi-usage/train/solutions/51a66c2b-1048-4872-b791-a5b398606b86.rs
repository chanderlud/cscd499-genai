use windows::core::{Error, Result};
use windows::Win32::UI::HiDpi::{
    AreDpiAwarenessContextsEqual, DPI_AWARENESS_CONTEXT_SYSTEM_AWARE, DPI_AWARENESS_CONTEXT_UNAWARE,
};

fn call_are_dpi_awareness_contexts_equal() -> Result<windows::core::BOOL> {
    // SAFETY: AreDpiAwarenessContextsEqual is safe to call with valid DPI_AWARENESS_CONTEXT constants.
    let result = unsafe {
        AreDpiAwarenessContextsEqual(
            DPI_AWARENESS_CONTEXT_UNAWARE,
            DPI_AWARENESS_CONTEXT_SYSTEM_AWARE,
        )
    };
    Ok(result)
}
