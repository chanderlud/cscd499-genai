use windows::core::Error;
use windows::Win32::Foundation::WIN32_ERROR;
use windows::Win32::Globalization::{CompareStringEx, COMPARESTRING_RESULT, COMPARE_STRING_FLAGS};

fn call_compare_string_ex() -> WIN32_ERROR {
    let s1: Vec<u16> = "hello".encode_utf16().chain(std::iter::once(0)).collect();
    let s2: Vec<u16> = "hello".encode_utf16().chain(std::iter::once(0)).collect();

    let result = unsafe {
        CompareStringEx(
            windows::core::w!("en-US"),
            COMPARE_STRING_FLAGS(0),
            &s1,
            &s2,
            None,
            None,
            None,
        )
    };

    if result == COMPARESTRING_RESULT(0) {
        WIN32_ERROR(Error::from_thread().code().0 as u32)
    } else {
        WIN32_ERROR(0)
    }
}
