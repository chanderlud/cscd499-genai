use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows::core::{Error, Result};
use windows::Win32::Globalization::{CompareStringOrdinal, COMPARESTRING_RESULT};

fn wide_null(s: &OsStr) -> Vec<u16> {
    use std::iter::once;
    s.encode_wide().chain(once(0)).collect()
}

fn call_compare_string_ordinal() -> Result<COMPARESTRING_RESULT> {
    let s1 = wide_null(OsStr::new("Hello"));
    let s2 = wide_null(OsStr::new("World"));

    // SAFETY: CompareStringOrdinal is called with valid, null-terminated wide string slices.
    let result = unsafe { CompareStringOrdinal(&s1, &s2, false) };

    if result.0 == 0 {
        Err(Error::from_thread())
    } else {
        Ok(result)
    }
}
