#[allow(unused_imports)]
use windows::core::{Error, Result, HRESULT};
use windows::Win32::Globalization::{CompareStringEx, COMPARE_STRING_FLAGS, LOCALE_NAME_INVARIANT};

fn call_compare_string_ex() -> HRESULT {
    let s1 = [0u16];
    let s2 = [0u16];
    // SAFETY: CompareStringEx is safe to call with valid locale and string pointers.
    let result = unsafe {
        CompareStringEx(
            LOCALE_NAME_INVARIANT,
            COMPARE_STRING_FLAGS(0),
            &s1,
            &s2,
            None,
            None,
            None,
        )
    };
    if result.0 == 0 {
        Error::from_thread().code()
    } else {
        HRESULT::from_win32(0)
    }
}
