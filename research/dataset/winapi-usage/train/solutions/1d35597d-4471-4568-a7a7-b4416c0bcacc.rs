use windows::core::{Error, Result};
use windows::Win32::Globalization::{CompareStringA, COMPARESTRING_RESULT};

fn call_compare_string_a() -> Result<COMPARESTRING_RESULT> {
    let s1: &[i8] = &[0x48, 0x69, 0];
    let s2: &[i8] = &[0x48, 0x69, 0];

    // SAFETY: CompareStringA is called with valid null-terminated string slices.
    let result = unsafe { CompareStringA(0x0400, 0, s1, s2) };
    if result.0 == 0 {
        return Err(Error::from_thread());
    }
    Ok(result)
}
