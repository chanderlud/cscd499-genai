use windows::core::{Error, Result};
use windows::Win32::Globalization::{CompareStringEx, COMPARESTRING_RESULT, COMPARE_STRING_FLAGS};

fn call_compare_string_ex() -> Result<COMPARESTRING_RESULT> {
    let result = unsafe {
        CompareStringEx(
            windows::core::w!("en-US"),
            COMPARE_STRING_FLAGS(0),
            &[0x0061, 0x0062, 0x0063, 0],
            &[0x0061, 0x0062, 0x0063, 0],
            None,
            None,
            None,
        )
    };
    if result.0 == 0 {
        return Err(Error::from_thread());
    }
    Ok(result)
}
