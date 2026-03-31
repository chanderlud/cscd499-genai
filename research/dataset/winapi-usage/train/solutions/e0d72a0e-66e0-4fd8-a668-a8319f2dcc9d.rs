use windows::core::{Error, Result};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Globalization::CompareStringOrdinal;

fn call_compare_string_ordinal() -> WIN32_ERROR {
    // SAFETY: CompareStringOrdinal is called with valid, null-terminated string slices.
    unsafe {
        let _ = CompareStringOrdinal(&[0], &[0], false);
        WIN32_ERROR(0)
    }
}
